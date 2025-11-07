# Vault Production Configuration
# Phase 6: Dual listener (HTTPS external, HTTP internal for vaultrs compatibility)

ui = true

# Listener 1: HTTPS for external access (localhost:8200)
listener "tcp" {
  address       = "0.0.0.0:8200"
  tls_cert_file = "/vault/certs/vault.crt"
  tls_key_file  = "/vault/certs/vault-key.pem"
  tls_disable   = false
}

# Listener 2: HTTP for internal Docker network (vault:8201)
# Note: vaultrs 0.7.x doesn't support TLS skip, so internal services use HTTP
listener "tcp" {
  address     = "0.0.0.0:8201"
  tls_disable = true
}

# Storage backend: Raft (production-ready, HA-capable)
# Integrated storage with automatic leader election
storage "raft" {
  path    = "/vault/raft"
  node_id = "vault-ce-node1"
}

# API address (HTTP for internal, HTTPS for external)
api_addr = "http://vault:8201"
cluster_addr = "https://vault:8202"

# Disable mlock for containerized environments
disable_mlock = true
