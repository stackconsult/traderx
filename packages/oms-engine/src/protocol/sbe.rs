use super::{OrderProtocol, OrderFrame, ProtocolError};
use crate::state_machine::{Order, OrderType, Side};
use rust_decimal::Decimal;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use byteorder::{ReadBytesExt, WriteBytesExt, LittleEndian};
use std::io::{Cursor, Write};

/// Simple Binary Encoding (SBE) Protocol Implementation
/// Optimized for high-performance order transmission
pub struct SBEProtocol {
    /// Protocol version
    version: u16,
    
    /// Template ID for orders
    order_template_id: u16,
}

impl SBEProtocol {
    /// Create new SBE protocol instance
    pub fn new() -> Result<Self, ProtocolError> {
        Ok(Self {
            version: 1,
            order_template_id: 100,
        })
    }
    
    /// Encode string with SBE var-length encoding
    fn encode_string(&self, buffer: &mut Vec<u8>, s: &str, max_length: usize) -> Result<(), ProtocolError> {
        if s.len() > max_length {
            return Err(ProtocolError::Encoding(format!("String too long: {} > {}", s.len(), max_length)));
        }
        
        // Write length as var uint
        self.write_var_uint(buffer, s.len() as u64)?;
        
        // Write string bytes
        buffer.extend_from_slice(s.as_bytes());
        
        Ok(())
    }
    
    /// Decode string with SBE var-length encoding
    fn decode_string(&self, cursor: &mut Cursor<&[u8]>, max_length: usize) -> Result<String, ProtocolError> {
        let length = self.read_var_uint(cursor)? as usize;
        
        if length > max_length {
            return Err(ProtocolError::Decoding(format!("String too long: {} > {}", length, max_length)));
        }
        
        let mut bytes = vec![0u8; length];
        cursor.read_exact(&mut bytes)
            .map_err(|e| ProtocolError::Decoding(e.to_string()))?;
        
        String::from_utf8(bytes)
            .map_err(|e| ProtocolError::Decoding(e.to_string()))
    }
    
    /// Write variable-length uint
    fn write_var_uint(&self, buffer: &mut Vec<u8>, mut value: u64) -> Result<(), ProtocolError> {
        loop {
            let mut byte = (value & 0x7F) as u8;
            value >>= 7;
            
            if value != 0 {
                byte |= 0x80;
            }
            
            buffer.push(byte);
            
            if value == 0 {
                break;
            }
        }
        
        Ok(())
    }
    
    /// Read variable-length uint
    fn read_var_uint(&self, cursor: &mut Cursor<&[u8]>) -> Result<u64, ProtocolError> {
        let mut result = 0u64;
        let mut shift = 0;
        
        loop {
            let byte = cursor.read_u8()
                .map_err(|e| ProtocolError::Decoding(e.to_string()))?;
            
            result |= ((byte & 0x7F) as u64) << shift;
            
            if byte & 0x80 == 0 {
                break;
            }
            
            shift += 7;
            if shift >= 64 {
                return Err(ProtocolError::Decoding("Var uint too large".to_string()));
            }
        }
        
        Ok(result)
    }
    
    /// Encode decimal as scaled integer
    fn encode_decimal(&self, buffer: &mut Vec<u8>, decimal: &Decimal, scale: i8) -> Result<(), ProtocolError> {
        let scaled = (decimal * Decimal::new(10i64.pow(scale.abs() as u32), 0))
            .to_string()
            .parse::<i64>()
            .map_err(|e| ProtocolError::Encoding(e.to_string()))?;
        
        buffer.write_i64::<LittleEndian>(scaled)
            .map_err(|e| ProtocolError::Encoding(e.to_string()))?;
        
        Ok(())
    }
    
    /// Decode decimal from scaled integer
    fn decode_decimal(&self, cursor: &mut Cursor<&[u8]>, scale: i8) -> Result<Decimal, ProtocolError> {
        let scaled = cursor.read_i64::<LittleEndian>()
            .map_err(|e| ProtocolError::Decoding(e.to_string()))?;
        
        let divisor = Decimal::new(10i64.pow(scale.abs() as u32), 0);
        Decimal::new(scaled, 0) / divisor
            .map_err(|e| ProtocolError::Decoding(e.to_string()))
    }
}

impl OrderProtocol for SBEProtocol {
    fn encode_order(&self, order: &Order) -> Result<Vec<u8>, ProtocolError> {
        let mut buffer = Vec::with_capacity(128);
        
        // Message header
        buffer.write_u16::<LittleEndian>(self.order_template_id)
            .map_err(|e| ProtocolError::Encoding(e.to_string()))?;
        buffer.write_u16::<LittleEndian>(self.version)
            .map_err(|e| ProtocolError::Encoding(e.to_string()))?;
        buffer.write_u32::<LittleEndian>(0) // Block length (placeholder)
            .map_err(|e| ProtocolError::Encoding(e.to_string()))?;
        
        // Order fields
        buffer.write_u8(match order.side { Side::Buy => 1, Side::Sell => 2 })
            .map_err(|e| ProtocolError::Encoding(e.to_string()))?;
        
        buffer.write_u8(match order.order_type {
            OrderType::Market => 1,
            OrderType::Limit => 2,
            OrderType::Stop => 3,
            OrderType::StopLimit => 4,
        })
            .map_err(|e| ProtocolError::Encoding(e.to_string()))?;
        
        // UUID as bytes
        buffer.extend_from_slice(order.order_id.as_bytes());
        buffer.extend_from_slice(order.account_id.as_bytes());
        
        // Symbol (var-length, max 32 chars)
        self.encode_string(&mut buffer, &order.symbol, 32)?;
        
        // Quantity (scaled decimal, scale 8)
        self.encode_decimal(&mut buffer, &order.original_quantity, 8)?;
        
        // Timestamp (Unix timestamp in nanoseconds)
        let timestamp_nanos = order.created_at.timestamp_nanos_opt()
            .ok_or_else(|| ProtocolError::Encoding("Invalid timestamp".to_string()))?;
        buffer.write_u64::<LittleEndian>(timestamp_nanos as u64)
            .map_err(|e| ProtocolError::Encoding(e.to_string()))?;
        
        // Update block length
        let block_length = (buffer.len() - 8) as u32; // Subtract header size
        buffer[4..8].copy_from_slice(&block_length.to_le_bytes());
        
        // Calculate checksum (simple sum)
        let checksum: u32 = buffer.iter().map(|&b| b as u32).sum();
        buffer.write_u32::<LittleEndian>(checksum)
            .map_err(|e| ProtocolError::Encoding(e.to_string()))?;
        
        Ok(buffer)
    }
    
    fn decode_order(&self, data: &[u8]) -> Result<Order, ProtocolError> {
        if data.len() < 24 {
            return Err(ProtocolError::Decoding("Data too short".to_string()));
        }
        
        let mut cursor = Cursor::new(data);
        
        // Read header
        let template_id = cursor.read_u16::<LittleEndian>()
            .map_err(|e| ProtocolError::Decoding(e.to_string()))?;
        let version = cursor.read_u16::<LittleEndian>()
            .map_err(|e| ProtocolError::Decoding(e.to_string()))?;
        let _block_length = cursor.read_u32::<LittleEndian>()
            .map_err(|e| ProtocolError::Decoding(e.to_string()))?;
        
        if template_id != self.order_template_id {
            return Err(ProtocolError::Decoding("Invalid template ID".to_string()));
        }
        
        // Read order fields
        let side_byte = cursor.read_u8()
            .map_err(|e| ProtocolError::Decoding(e.to_string()))?;
        let side = match side_byte {
            1 => Side::Buy,
            2 => Side::Sell,
            _ => return Err(ProtocolError::Decoding("Invalid side".to_string())),
        };
        
        let type_byte = cursor.read_u8()
            .map_err(|e| ProtocolError::Decoding(e.to_string()))?;
        let order_type = match type_byte {
            1 => OrderType::Market,
            2 => OrderType::Limit,
            3 => OrderType::Stop,
            4 => OrderType::StopLimit,
            _ => return Err(ProtocolError::Decoding("Invalid order type".to_string())),
        };
        
        // Read UUIDs
        let mut order_id_bytes = [0u8; 16];
        cursor.read_exact(&mut order_id_bytes)
            .map_err(|e| ProtocolError::Decoding(e.to_string()))?;
        let order_id = Uuid::from_bytes(order_id_bytes);
        
        let mut account_id_bytes = [0u8; 16];
        cursor.read_exact(&mut account_id_bytes)
            .map_err(|e| ProtocolError::Decoding(e.to_string()))?;
        let account_id = Uuid::from_bytes(account_id_bytes);
        
        // Read symbol
        let symbol = self.decode_string(&mut cursor, 32)?;
        
        // Read quantity
        let quantity = self.decode_decimal(&mut cursor, 8)?;
        
        // Read timestamp
        let timestamp_nanos = cursor.read_u64::<LittleEndian>()
            .map_err(|e| ProtocolError::Decoding(e.to_string()))?;
        let created_at = DateTime::from_timestamp(
            (timestamp_nanos / 1_000_000_000) as i64,
            (timestamp_nanos % 1_000_000_000) as u32
        ).ok_or_else(|| ProtocolError::Decoding("Invalid timestamp".to_string()))?;
        
        // Verify checksum
        let data_without_checksum = &data[..data.len() - 4];
        let expected_checksum: u32 = data_without_checksum.iter().map(|&b| b as u32).sum();
        let actual_checksum = cursor.read_u32::<LittleEndian>()
            .map_err(|e| ProtocolError::Decoding(e.to_string()))?;
        
        if expected_checksum != actual_checksum {
            return Err(ProtocolError::Decoding("Checksum mismatch".to_string()));
        }
        
        Ok(Order {
            order_id,
            account_id,
            symbol,
            side,
            order_type,
            original_quantity: quantity,
            state: crate::state_machine::OrderState::New,
            created_at,
            updated_at: created_at,
        })
    }
    
    fn name(&self) -> &'static str {
        "SBE"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    
    #[tokio::test]
    fn test_sbe_encode_decode_roundtrip() {
        let protocol = SBEProtocol::new().unwrap();
        
        let order = Order {
            order_id: Uuid::new_v4(),
            account_id: Uuid::new_v4(),
            symbol: "BTCUSDT".to_string(),
            side: Side::Buy,
            order_type: OrderType::Limit,
            original_quantity: Decimal::from_str_exact("100.12345678").unwrap(),
            state: crate::state_machine::OrderState::New,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        // Encode
        let encoded = protocol.encode_order(&order).unwrap();
        assert!(!encoded.is_empty());
        
        // Decode
        let decoded = protocol.decode_order(&encoded).unwrap();
        
        // Verify
        assert_eq!(decoded.order_id, order.order_id);
        assert_eq!(decoded.account_id, order.account_id);
        assert_eq!(decoded.symbol, order.symbol);
        assert_eq!(decoded.side, order.side);
        assert_eq!(decoded.order_type, order.order_type);
        assert_eq!(decoded.original_quantity, order.original_quantity);
    }
}
