"""
Know Your Agent (KYA) & Behavioral Binding (Task 4.2)
Bridges the anonymity gap by linking agentic tool-calls to verified human intent.
Integrates with Sumsub AI Agent Verification SDK.
"""

import asyncio
import json
import time
import os
import hmac
from datetime import datetime, timedelta
from typing import Dict, Optional, Tuple, Any
from dataclasses import dataclass, asdict
import httpx
import hashlib
import jwt
from cryptography.hazmat.primitives import hashes, serialization
from cryptography.hazmat.primitives.asymmetric import padding, rsa
import redis.asyncio as redis
import structlog

logger = structlog.get_logger(__name__)


@dataclass
class AgentIdentity:
    """Agent identity with KYA binding."""
    agent_id: str
    agent_did: str  # Decentralized Identifier
    human_principal_id: str
    human_did: str
    verification_level: str  # "BASIC", "ENHANCED", "PROFESSIONAL"
    certificate_chain: str
    liveness_required: bool
    risk_score: float
    created_at: datetime
    expires_at: datetime
    metadata: Dict[str, Any]


@dataclass
class KYACertificate:
    """KYA certificate linking agent to human."""
    certificate_id: str
    agent_id: str
    human_id: str
    signature: str
    public_key: str
    issued_at: datetime
    expires_at: datetime
    permissions: list[str]
    constraints: Dict[str, Any]


class SumsubKYAProvider:
    """Sumsub AI Agent Verification SDK integration."""
    
    def __init__(self, 
                 api_token: str,
                 api_secret: str,
                 base_url: str = "https://api.sumsub.com"):
        self.api_token = api_token
        self.api_secret = api_secret
        self.base_url = base_url
        self.client = httpx.AsyncClient(timeout=30.0)
        
    async def verify_agent_binding(self, 
                                  agent_did: str,
                                  human_did: str) -> Optional[AgentIdentity]:
        """
        Verify agent-to-human binding via Sumsub.
        
        Args:
            agent_did: Agent's Decentralized Identifier
            human_did: Human's Decentralized Identifier
            
        Returns:
            AgentIdentity if verified, None otherwise
        """
        try:
            # Prepare request
            endpoint = "/resources/ai/agents/verifyBinding"
            payload = {
                "agentDid": agent_did,
                "humanDid": human_did,
                "timestamp": datetime.utcnow().isoformat()
            }
            
            # Sign request
            signature = self._sign_request(payload)
            
            # Make API call
            response = await self.client.post(
                f"{self.base_url}{endpoint}",
                json=payload,
                headers={
                    "X-App-Token": self.api_token,
                    "X-Signature": signature
                }
            )
            
            if response.status_code == 200:
                data = response.json()
                
                if data.get("verified"):
                    return AgentIdentity(
                        agent_id=data["agentId"],
                        agent_did=agent_did,
                        human_principal_id=data["humanId"],
                        human_did=human_did,
                        verification_level=data["verificationLevel"],
                        certificate_chain=json.dumps(data["certificateChain"]),
                        liveness_required=data["livenessRequired"],
                        risk_score=data["riskScore"],
                        created_at=datetime.fromisoformat(data["createdAt"]),
                        expires_at=datetime.fromisoformat(data["expiresAt"]),
                        metadata=data.get("metadata", {})
                    )
            
            logger.warning(
                "Agent verification failed",
                agent_did=agent_did,
                human_did=human_did,
                status_code=response.status_code,
                response=response.text
            )
            return None
            
        except Exception as e:
            logger.error(
                "Failed to verify agent binding",
                agent_did=agent_did,
                human_did=human_did,
                error=str(e)
            )
            return None
    
    async def trigger_liveness_test(self, 
                                   human_id: str,
                                   reason: str) -> bool:
        """
        Trigger targeted liveness test for human.
        
        Args:
            human_id: Human principal ID
            reason: Reason for liveness test
            
        Returns:
            True if triggered successfully
        """
        try:
            endpoint = "/resources/ai/agents/triggerLiveness"
            payload = {
                "humanId": human_id,
                "reason": reason,
                "timestamp": datetime.utcnow().isoformat()
            }
            
            signature = self._sign_request(payload)
            
            response = await self.client.post(
                f"{self.base_url}{endpoint}",
                json=payload,
                headers={
                    "X-App-Token": self.api_token,
                    "X-Signature": signature
                }
            )
            
            return response.status_code == 200
            
        except Exception as e:
            logger.error(
                "Failed to trigger liveness test",
                human_id=human_id,
                reason=reason,
                error=str(e)
            )
            return False
    
    def _sign_request(self, payload: Dict[str, Any]) -> str:
        """Sign request with API secret."""
        payload_str = json.dumps(payload, sort_keys=True, separators=(',', ':'))
        signature = hmac.new(
            self.api_secret.encode(),
            payload_str.encode(),
            hashlib.sha256
        ).hexdigest()
        return signature


class KYABindingManager:
    """
    Manages KYA bindings and validates agent-to-human relationships.
    """
    
    def __init__(self, 
                 sumsub_provider: SumsubKYAProvider,
                 redis_url: str):
        self.sumsub = sumsub_provider
        self.redis_url = redis_url
        self.redis_client = None
        
        # RSA key pair for certificate signing
        self.private_key = None
        self.public_key = None
        self._generate_key_pair()
        
        # Statistics
        self.stats = {
            "verifications": 0,
            "successful": 0,
            "failed": 0,
            "liveness_triggered": 0,
            "high_value_blocked": 0
        }
        
        logger.info("KYA Binding Manager initialized")
    
    def _generate_key_pair(self):
        """Generate RSA key pair for certificate signing."""
        self.private_key = rsa.generate_private_key(
            public_exponent=65537,
            key_size=2048
        )
        self.public_key = self.private_key.public_key()
    
    async def verify_handoff_kya(self, 
                                agent_id: str,
                                agent_did: str,
                                operation_type: str,
                                operation_value: float) -> Tuple[bool, Optional[AgentIdentity]]:
        """
        Verify KYA for a handoff operation.
        
        Args:
            agent_id: Agent ID
            agent_did: Agent DID
            operation_type: Type of operation
            operation_value: Value of operation (for risk assessment)
            
        Returns:
            (is_allowed, agent_identity)
        """
        self.stats["verifications"] += 1
        
        # Check cache first
        cached_identity = await self._get_cached_identity(agent_did)
        if cached_identity:
            # Check if certificate is still valid
            if cached_identity.expires_at > datetime.utcnow():
                # Check if liveness is required for high-value operation
                if operation_value > 10000 and cached_identity.liveness_required:
                    if not await self._verify_recent_liveness(cached_identity.human_principal_id):
                        await self._trigger_liveness_for_high_value(
                            cached_identity.human_principal_id,
                            operation_type,
                            operation_value
                        )
                        self.stats["high_value_blocked"] += 1
                        return False, None
                
                self.stats["successful"] += 1
                return True, cached_identity
        
        # Verify with Sumsub
        # In production, we'd get human_did from the agent's metadata
        human_did = await self._get_human_did_for_agent(agent_id)
        if not human_did:
            logger.error(
                "No human DID found for agent",
                agent_id=agent_id
            )
            self.stats["failed"] += 1
            return False, None
        
        # Verify binding
        identity = await self.sumsub.verify_agent_binding(agent_did, human_did)
        if identity:
            # Cache the identity
            await self._cache_identity(identity)
            
            # Generate KYA certificate
            certificate = await self._generate_kya_certificate(identity)
            
            # Log successful verification
            logger.info(
                "KYA verification successful",
                agent_id=agent_id,
                human_id=identity.human_principal_id,
                verification_level=identity.verification_level,
                certificate_id=certificate.certificate_id
            )
            
            self.stats["successful"] += 1
            return True, identity
        else:
            self.stats["failed"] += 1
            return False, None
    
    async def _get_cached_identity(self, agent_did: str) -> Optional[AgentIdentity]:
        """Get cached agent identity."""
        if not self.redis_client:
            self.redis_client = redis.from_url(self.redis_url)
        
        try:
            key = f"kya:identity:{agent_did}"
            data = await self.redis_client.get(key)
            
            if data:
                identity_dict = json.loads(data)
                return AgentIdentity(**identity_dict)
                
        except Exception as e:
            logger.error(
                "Failed to get cached identity",
                agent_did=agent_did,
                error=str(e)
            )
        
        return None
    
    async def _cache_identity(self, identity: AgentIdentity):
        """Cache agent identity."""
        if not self.redis_client:
            self.redis_client = redis.from_url(self.redis_url)
        
        try:
            key = f"kya:identity:{identity.agent_did}"
            data = json.dumps(asdict(identity), default=str)
            
            # Cache until expiry
            ttl = int((identity.expires_at - datetime.utcnow()).total_seconds())
            await self.redis_client.setex(key, ttl, data)
            
        except Exception as e:
            logger.error(
                "Failed to cache identity",
                agent_did=identity.agent_did,
                error=str(e)
            )
    
    async def _generate_kya_certificate(self, identity: AgentIdentity) -> KYACertificate:
        """Generate KYA certificate for the agent."""
        certificate_data = {
            "agentId": identity.agent_id,
            "humanId": identity.human_principal_id,
            "issuedAt": datetime.utcnow().isoformat(),
            "expiresAt": identity.expires_at.isoformat(),
            "permissions": self._get_permissions_for_level(identity.verification_level),
            "riskScore": identity.risk_score
        }
        
        # Sign certificate
        data_str = json.dumps(certificate_data, sort_keys=True)
        signature = self.private_key.sign(
            data_str.encode(),
            padding.PSS(
                mgf=padding.MGF1(hashes.SHA256()),
                salt_length=padding.PSS.MAX_LENGTH
            ),
            hashes.SHA256()
        )
        
        # Get public key in PEM format
        public_key_pem = self.public_key.public_bytes(
            encoding=serialization.Encoding.PEM,
            format=serialization.PublicFormat.SubjectPublicKeyInfo
        ).decode()
        
        certificate = KYACertificate(
            certificate_id=f"kya-{int(time.time())}-{identity.agent_id[:8]}",
            agent_id=identity.agent_id,
            human_id=identity.human_principal_id,
            signature=signature.hex(),
            public_key=public_key_pem,
            issued_at=datetime.utcnow(),
            expires_at=identity.expires_at,
            permissions=certificate_data["permissions"],
            constraints={
                "max_daily_volume": self._get_max_volume_for_risk(identity.risk_score),
                "requires_liveness": identity.liveness_required,
                "verification_level": identity.verification_level
            }
        )
        
        # Store certificate
        await self._store_certificate(certificate)
        
        return certificate
    
    async def _store_certificate(self, certificate: KYACertificate):
        """Store KYA certificate."""
        if not self.redis_client:
            self.redis_client = redis.from_url(self.redis_url)
        
        try:
            key = f"kya:certificate:{certificate.certificate_id}"
            data = json.dumps(asdict(certificate), default=str)
            
            ttl = int((certificate.expires_at - datetime.utcnow()).total_seconds())
            await self.redis_client.setex(key, ttl, data)
            
        except Exception as e:
            logger.error(
                "Failed to store certificate",
                certificate_id=certificate.certificate_id,
                error=str(e)
            )
    
    def _get_permissions_for_level(self, level: str) -> list[str]:
        """Get permissions based on verification level."""
        permissions = {
            "BASIC": ["read", "trade_small"],
            "ENHANCED": ["read", "trade_small", "trade_medium", "withdraw_small"],
            "PROFESSIONAL": ["read", "trade_small", "trade_medium", "trade_large", "withdraw_medium", "api_access"]
        }
        return permissions.get(level, [])
    
    def _get_max_volume_for_risk(self, risk_score: float) -> float:
        """Get maximum daily volume based on risk score."""
        if risk_score < 0.3:
            return 100000.0
        elif risk_score < 0.7:
            return 50000.0
        else:
            return 10000.0
    
    async def _verify_recent_liveness(self, human_id: str) -> bool:
        """Check if human has recent liveness verification."""
        if not self.redis_client:
            self.redis_client = redis.from_url(self.redis_url)
        
        try:
            key = f"kya:liveness:{human_id}"
            last_liveness = await self.redis_client.get(key)
            
            if last_liveness:
                last_time = datetime.fromisoformat(last_liveness)
                # Liveness valid for 24 hours
                if datetime.utcnow() - last_time < timedelta(hours=24):
                    return True
                    
        except Exception as e:
            logger.error(
                "Failed to verify liveness",
                human_id=human_id,
                error=str(e)
            )
        
        return False
    
    async def _trigger_liveness_for_high_value(self, 
                                             human_id: str,
                                             operation_type: str,
                                             operation_value: float):
        """Trigger liveness test for high-value operation."""
        reason = f"High-value {operation_type} operation: ${operation_value:,.2f}"
        
        success = await self.sumsub.trigger_liveness_test(human_id, reason)
        if success:
            self.stats["liveness_triggered"] += 1
            logger.warning(
                "Liveness test triggered for high-value operation",
                human_id=human_id,
                operation_type=operation_type,
                value=operation_value
            )
    
    async def _get_human_did_for_agent(self, agent_id: str) -> Optional[str]:
        """Get human DID associated with agent."""
        # In production, this would come from agent registration
        # For now, return a mock mapping
        agent_human_map = {
            "agent-001": "did:human:001",
            "agent-002": "did:human:002",
            "claude-4.6": "did:human:admin",
            "gemma-4": "did:human:admin"
        }
        return agent_human_map.get(agent_id)
    
    def get_statistics(self) -> Dict[str, Any]:
        """Get KYA binding statistics."""
        return self.stats.copy()
    
    def save_kya_handshake(self, filepath: str, certificate: KYACertificate):
        """Save KYA handshake proof artifact."""
        handshake = {
            "timestamp": datetime.utcnow().isoformat(),
            "certificate": asdict(certificate),
            "verification_hash": hashlib.sha256(
                json.dumps(asdict(certificate), sort_keys=True).encode()
            ).hexdigest(),
            "cryptographic_link": {
                "agent_id": certificate.agent_id,
                "human_id": certificate.human_id,
                "signature": certificate.signature,
                "public_key": certificate.public_key
            }
        }
        
        with open(filepath, 'w') as f:
            json.dump(handshake, f, indent=2)
        
        logger.info(
            "KYA handshake saved",
            filepath=filepath,
            certificate_id=certificate.certificate_id
        )


# Global KYA manager instance
kya_manager = None


async def get_kya_manager() -> KYABindingManager:
    """Get the global KYA manager instance."""
    global kya_manager
    if not kya_manager:
        # Initialize with environment variables
        api_token = os.getenv("SUMSUB_API_TOKEN")
        api_secret = os.getenv("SUMSUB_API_SECRET")
        redis_url = os.getenv("REDIS_URL", "redis://localhost:6379")
        
        if not api_token or not api_secret:
            raise ValueError("SUMSUB_API_TOKEN and SUMSUB_API_SECRET must be set")
        
        sumsub_provider = SumsubKYAProvider(api_token, api_secret)
        kya_manager = KYABindingManager(sumsub_provider, redis_url)
    
    return kya_manager
