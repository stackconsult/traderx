use super::{OrderProtocol, OrderFrame, ProtocolError};
use crate::state_machine::{Order, OrderType, Side};
use rust_decimal::Decimal;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use byteorder::{ReadBytesExt, WriteBytesExt, BigEndian};
use std::io::{Cursor, Read, Write};

/// ITCH Protocol Implementation
/// Based on NASDAQ ITCH 5.0 specification for market data and order messages
pub struct ITCHProtocol {
    /// Protocol version
    version: u8,
    
    /// Message types
    add_order_msg: u8,
    order_executed_msg: u8,
    order_cancel_msg: u8,
}

impl ITCHProtocol {
    /// Create new ITCH protocol instance
    pub fn new() -> Result<Self, ProtocolError> {
        Ok(Self {
            version: 5,
            add_order_msg: b'A',
            order_executed_msg: b'E',
            order_cancel_msg: b'X',
        })
    }
    
    /// Write ITCH timestamp (nanoseconds since midnight)
    fn write_timestamp(&self, buffer: &mut Vec<u8>, timestamp: DateTime<Utc>) -> Result<(), ProtocolError> {
        let midnight = timestamp.date_naive().and_hms_opt(0, 0, 0)
            .map(|dt| dt.and_utc())
            .ok_or_else(|| ProtocolError::Encoding("Invalid timestamp".to_string()))?;
        
        let nanos_since_midnight = timestamp.signed_duration_since(midnight).num_nanoseconds()
            .ok_or_else(|| ProtocolError::Encoding("Invalid duration".to_string()))?;
        
        buffer.write_u64::<BigEndian>(nanos_since_midnight as u64)
            .map_err(|e| ProtocolError::Encoding(e.to_string()))?;
        
        Ok(())
    }
    
    /// Read ITCH timestamp
    fn read_timestamp(&self, cursor: &mut Cursor<&[u8]>, date: chrono::NaiveDate) -> Result<DateTime<Utc>, ProtocolError> {
        let nanos_since_midnight = cursor.read_u64::<BigEndian>()
            .map_err(|e| ProtocolError::Decoding(e.to_string()))?;
        
        let midnight = date.and_hms_opt(0, 0, 0)
            .map(|dt| dt.and_utc())
            .ok_or_else(|| ProtocolError::Decoding("Invalid date".to_string()))?;
        
        let timestamp = midnight + chrono::Duration::nanoseconds(nanos_since_midnight as i64);
        
        Ok(timestamp)
    }
    
    /// Write price as integer (scaled by 10000 for 4 decimal places)
    fn write_price(&self, buffer: &mut Vec<u8>, price: Decimal) -> Result<(), ProtocolError> {
        let scaled = (price * Decimal::new(10000, 0))
            .to_string()
            .parse::<u64>()
            .map_err(|e| ProtocolError::Encoding(e.to_string()))?;
        
        buffer.write_u64::<BigEndian>(scaled)
            .map_err(|e| ProtocolError::Encoding(e.to_string()))?;
        
        Ok(())
    }
    
    /// Read price from integer
    fn read_price(&self, cursor: &mut Cursor<&[u8]>) -> Result<Decimal, ProtocolError> {
        let scaled = cursor.read_u64::<BigEndian>()
            .map_err(|e| ProtocolError::Decoding(e.to_string()))?;
        
        Ok(Decimal::new(scaled as i64, 4u32))
    }
    
    /// Pad string to fixed length
    fn pad_string(&self, s: &str, length: usize) -> String {
        format!("{:width$}", s, width = length).replace(' ', " ")
    }
    
    /// Unpad string
    fn unpad_string(&self, s: &str) -> String {
        s.trim_end().to_string()
    }
}

impl OrderProtocol for ITCHProtocol {
    fn encode_order(&self, order: &Order) -> Result<Vec<u8>, ProtocolError> {
        let mut buffer = Vec::with_capacity(64);
        
        // Message header
        buffer.push(self.add_order_msg); // Message type
        buffer.write_u16::<BigEndian>(0) // Message length (placeholder)
            .map_err(|e| ProtocolError::Encoding(e.to_string()))?;
        
        // Timestamp
        self.write_timestamp(&mut buffer, order.created_at)?;
        
        // Order reference number (UUID as u64 hash)
        let order_ref = u64::from_be_bytes(
            order.order_id.as_bytes()[..8].try_into()
                .map_err(|_| ProtocolError::Encoding("Invalid UUID".to_string()))?
        );
        buffer.write_u64::<BigEndian>(order_ref)
            .map_err(|e| ProtocolError::Encoding(e.to_string()))?;
        
        // Buy/Sell indicator
        buffer.push(match order.side {
            Side::Buy => b'B',
            Side::Sell => b'S',
        });
        
        // Number of shares (quantity)
        let shares = order.original_quantity.to_string()
            .parse::<u64>()
            .map_err(|e| ProtocolError::Encoding(e.to_string()))?;
        buffer.write_u32::<BigEndian>(shares as u32)
            .map_err(|e| ProtocolError::Encoding(e.to_string()))?;
        
        // Symbol (fixed 8 bytes)
        let symbol = self.pad_string(&order.symbol, 8);
        buffer.extend_from_slice(symbol.as_bytes());
        
        // Price (for limit orders)
        if order.order_type == OrderType::Limit {
            self.write_price(&mut buffer, order.price.unwrap_or(Decimal::ZERO))?;
        } else {
            buffer.write_u64::<BigEndian>(0) // Market orders have no price
                .map_err(|e| ProtocolError::Encoding(e.to_string()))?;
        }
        
        // Attribution (4 bytes, using account ID hash)
        let attribution = u32::from_be_bytes(
            order.account_id.as_bytes()[..4].try_into()
                .map_err(|_| ProtocolError::Encoding("Invalid account ID".to_string()))?
        );
        buffer.write_u32::<BigEndian>(attribution)
            .map_err(|e| ProtocolError::Encoding(e.to_string()))?;
        
        // Update message length
        let length = (buffer.len() - 3) as u16; // Subtract type and length fields
        buffer[1..3].copy_from_slice(&length.to_be_bytes());
        
        Ok(buffer)
    }
    
    fn decode_order(&self, data: &[u8]) -> Result<Order, ProtocolError> {
        if data.len() < 45 {
            return Err(ProtocolError::Decoding("Data too short".to_string()));
        }
        
        let mut cursor = Cursor::new(data);
        
        // Read header
        let msg_type = cursor.read_u8()
            .map_err(|e| ProtocolError::Decoding(e.to_string()))?;
        
        if msg_type != self.add_order_msg {
            return Err(ProtocolError::Decoding("Invalid message type".to_string()));
        }
        
        let _msg_length = cursor.read_u16::<BigEndian>()
            .map_err(|e| ProtocolError::Decoding(e.to_string()))?;
        
        // Read timestamp
        let timestamp_nanos = cursor.read_u64::<BigEndian>()
            .map_err(|e| ProtocolError::Decoding(e.to_string()))?;
        
        // Use current date (ITCH doesn't encode date in messages)
        let date = Utc::now().date_naive();
        let created_at = self.read_timestamp(&mut cursor, date)?;
        
        // Read order reference
        let order_ref = cursor.read_u64::<BigEndian>()
            .map_err(|e| ProtocolError::Decoding(e.to_string()))?;
        
        // Reconstruct UUID from order reference (not ideal but works for demo)
        let mut order_id_bytes = [0u8; 16];
        order_id_bytes[..8].copy_from_slice(&order_ref.to_be_bytes());
        let order_id = Uuid::from_bytes(order_id_bytes);
        
        // Read side
        let side_byte = cursor.read_u8()
            .map_err(|e| ProtocolError::Decoding(e.to_string()))?;
        let side = match side_byte {
            b'B' => Side::Buy,
            b'S' => Side::Sell,
            _ => return Err(ProtocolError::Decoding("Invalid side indicator".to_string())),
        };
        
        // Read quantity
        let shares = cursor.read_u32::<BigEndian>()
            .map_err(|e| ProtocolError::Decoding(e.to_string()))?;
        let quantity = Decimal::new(shares as i64, 0);
        
        // Read symbol
        let mut symbol_bytes = [0u8; 8];
        cursor.read_exact(&mut symbol_bytes)
            .map_err(|e| ProtocolError::Decoding(e.to_string()))?;
        let symbol = self.unpad_string(&String::from_utf8_lossy(&symbol_bytes));
        
        // Read price
        let price = self.read_price(&mut cursor)?;
        
        // Read attribution
        let _attribution = cursor.read_u32::<BigEndian>()
            .map_err(|e| ProtocolError::Decoding(e.to_string()))?;
        
        // Determine order type based on price
        let order_type = if price == Decimal::ZERO {
            OrderType::Market
        } else {
            OrderType::Limit
        };
        
        Ok(Order {
            order_id,
            account_id: Uuid::new_v4(), // Can't reconstruct from attribution
            symbol,
            side,
            order_type,
            original_quantity: quantity,
            price: None,
            state: crate::state_machine::OrderState::New,
            created_at,
            updated_at: created_at,
        })
    }
    
    fn name(&self) -> &'static str {
        "ITCH"
    }
}

/// ITCH message types for market data
#[derive(Debug, Clone)]
pub enum ITCHMessage {
    /// Add Order message
    AddOrder {
        timestamp: DateTime<Utc>,
        order_ref: u64,
        side: Side,
        shares: u32,
        symbol: String,
        price: Decimal,
        attribution: u32,
    },
    
    /// Order Executed message
    OrderExecuted {
        timestamp: DateTime<Utc>,
        order_ref: u64,
        shares: u32,
        match_number: u64,
    },
    
    /// Order Cancel message
    OrderCancel {
        timestamp: DateTime<Utc>,
        order_ref: u64,
        canceled_shares: u32,
    },
    
    /// System Event message
    SystemEvent {
        timestamp: DateTime<Utc>,
        event_code: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    
    #[tokio::test]
    async fn test_itch_encode_decode_roundtrip() {
        let protocol = ITCHProtocol::new().unwrap();
        
        let order = Order {
            order_id: Uuid::new_v4(),
            account_id: Uuid::new_v4(),
            symbol: "AAPL".to_string(),
            side: Side::Buy,
            order_type: OrderType::Limit,
            original_quantity: Decimal::from(100),
            state: crate::state_machine::OrderState::New,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        // Encode
        let encoded = protocol.encode_order(&order).unwrap();
        assert!(!encoded.is_empty());
        assert_eq!(encoded[0], b'A'); // Add Order message type
        
        // Decode
        let decoded = protocol.decode_order(&encoded).unwrap();
        
        // Verify
        assert_eq!(decoded.symbol, order.symbol);
        assert_eq!(decoded.side, order.side);
        assert_eq!(decoded.original_quantity, order.original_quantity);
    }
}
