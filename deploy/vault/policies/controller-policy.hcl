# Vault Policy for Controller Service
# Phase 6 - AppRole Authentication
# Grants minimal permissions needed for profile signing/verification

# Transit engine - HMAC operations for profile signing
path "transit/hmac/profile-signing" {
  capabilities = ["create", "update"]
}

# Transit engine - Verify HMAC signatures  
path "transit/verify/profile-signing" {
  capabilities = ["create", "update"]
}

# Transit engine - Read key metadata (for monitoring)
path "transit/keys/profile-signing" {
  capabilities = ["read", "create", "update"]
}

# Allow token renewal (controller can extend its own token)
path "auth/token/renew-self" {
  capabilities = ["update"]
}

# Allow token lookup (controller can check its own token status)
path "auth/token/lookup-self" {
  capabilities = ["read"]
}
