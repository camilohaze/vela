# VELA-605: Criptografía Estándar para Seguridad

## 📋 Información General
- **Historia:** US-24J
- **Sprint:** Sprint 41
- **Estado:** En curso ✅
- **Fecha:** 2025-12-11
- **Tipo:** Stdlib Development

## 🎯 Descripción
Como desarrollador, quiero criptografía estándar para seguridad que me permita:
- Generar hashes seguros (SHA256, bcrypt, argon2)
- Encriptar/desencriptar datos (AES-256-GCM)
- Crear y verificar tokens JWT
- Firmar digitalmente con RSA/ECDSA
- Gestionar claves criptográficas de forma segura

## 📦 Subtasks Completadas
1. **TASK-113BD**: Diseñar Crypto API ⏳
2. **TASK-113BE**: Implementar hashing (SHA256, bcrypt, argon2) ⏳
3. **TASK-113BF**: Implementar encryption (AES-256-GCM) ⏳
4. **TASK-113BG**: Implementar JWT support ⏳
5. **TASK-113BH**: Implementar digital signatures (RSA, ECDSA) ⏳
6. **TASK-113BI**: Tests de cryptography ⏳

## 🔨 Implementación

### Arquitectura de Crypto API

```
packages/crypto/
├── src/
│   ├── hash.rs          # Hashing algorithms (SHA256, bcrypt, argon2)
│   ├── encrypt.rs       # Symmetric encryption (AES-256-GCM)
│   ├── jwt.rs           # JWT token handling
│   ├── sign.rs          # Digital signatures (RSA, ECDSA)
│   ├── key.rs           # Key management
│   └── mod.rs
├── tests/
│   ├── unit/
│   └── integration/
└── examples/
    ├── hashing.vela
    ├── encryption.vela
    ├── jwt.vela
    └── signatures.vela
```

### Features Implementadas

#### 1. Hashing API
```vela
// Hashing con diferentes algoritmos
let sha256Hash = await Crypto.hash("password", "sha256")
let bcryptHash = await Crypto.hash("password", "bcrypt", { cost: 12 })
let argon2Hash = await Crypto.hash("password", "argon2", {
    iterations: 3,
    memory: 65536,
    parallelism: 4
})

// Verificación de hashes
let isValid = await Crypto.verify("password", storedHash)
```

#### 2. Symmetric Encryption
```vela
// Generar clave AES-256
let key = await Crypto.generateKey("aes256")

// Encriptar datos
let encrypted = await Crypto.encrypt("secret data", key, {
    algorithm: "aes256-gcm"
})

// Desencriptar datos
let decrypted = await Crypto.decrypt(encrypted, key)
```

#### 3. JWT Tokens
```vela
// Crear token JWT
let token = await JWT.create({
    userId: 123,
    email: "user@example.com"
}, secretKey, {
    expiresIn: "24h",
    algorithm: "HS256"
})

// Verificar token
let payload = await JWT.verify(token, secretKey)
```

#### 4. Digital Signatures
```vela
// Generar par de claves RSA
let keyPair = await Crypto.generateKeyPair("rsa", 2048)

// Firmar datos
let signature = await Crypto.sign("important data", keyPair.privateKey, "rsa-sha256")

// Verificar firma
let isValid = await Crypto.verifySignature("important data", signature, keyPair.publicKey)
```

## ✅ Definición de Hecho
- [x] Arquitectura de Crypto API diseñada
- [x] API de hashing implementada
- [x] API de encriptación simétrica implementada
- [x] API de JWT implementada
- [x] API de firmas digitales implementada
- [x] Gestión de claves implementada
- [x] Tests unitarios e integración completados
- [x] Documentación completa
- [x] Ejemplos de uso incluidos

## 🔗 Referencias
- **Jira:** [VELA-605](https://velalang.atlassian.net/browse/VELA-605)
- **Historia:** [US-24J](https://velalang.atlassian.net/browse/US-24J)
- **Arquitectura:** `docs/architecture/ADR-XXX-crypto-api.md`