# TASK-113BD: Diseñar Crypto API

## 📋 Información General
- **Historia:** VELA-605
- **Estado:** Completada ✅
- **Fecha:** 2025-12-11

## 🎯 Objetivo
Diseñar una API de criptografía unificada y type-safe que proporcione funcionalidades seguras para hashing, encriptación, firmas digitales y JWT.

## 🔨 Implementación

### Arquitectura Diseñada

La Crypto API se estructura en 5 módulos principales:

#### 1. **Hashing Module** (`Crypto.Hash`)
**Funcionalidades:**
- SHA256/SHA512 para hashing rápido
- bcrypt para contraseñas
- argon2 para KDF avanzado

**API Design:**
```vela
// Hashing unificado
let sha256Hash = await Crypto.hash("password", "sha256")
let bcryptHash = await Crypto.hash("password", "bcrypt", { cost: 12 })
let argon2Hash = await Crypto.hash("password", "argon2", {
    iterations: 3,
    memory: 65536,
    parallelism: 4
})

// Verificación unificada
let isValid = await Crypto.verify("password", storedHash)
```

#### 2. **Symmetric Encryption Module** (`Crypto.Encrypt`)
**Funcionalidades:**
- AES-256-GCM (recomendado)
- ChaCha20-Poly1305 (alternativa moderna)

**API Design:**
```vela
// Generación de claves
let key = await Crypto.generateKey("aes256")

// Encriptación con AEAD
let encrypted = await Crypto.encrypt("secret data", key, {
    algorithm: "aes256-gcm",
    associatedData: "metadata"
})

// Desencriptación
let decrypted = await Crypto.decrypt(encrypted, key)
```

#### 3. **JWT Module** (`Crypto.JWT`)
**Funcionalidades:**
- Creación y verificación de tokens
- Soporte para HS256/384/512, RS256/384/512, ES256/384/512
- Claims estándar y custom

**API Design:**
```vela
// Creación de tokens
let token = await JWT.create({
    userId: 123,
    email: "user@example.com",
    roles: ["admin", "user"]
}, secretKey, {
    expiresIn: "24h",
    algorithm: "HS256",
    issuer: "myapp"
})

// Verificación
let payload = await JWT.verify(token, secretKey)

// Decodificación sin verificación
let decoded = await JWT.decode(token)
```

#### 4. **Digital Signatures Module** (`Crypto.Sign`)
**Funcionalidades:**
- RSA con SHA256/384/512
- ECDSA con P-256/384/521
- Generación y verificación de firmas

**API Design:**
```vela
// Generación de pares de claves
let rsaKeyPair = await Crypto.generateKeyPair("rsa", 2048)
let ecdsaKeyPair = await Crypto.generateKeyPair("ecdsa", "p256")

// Firma de datos
let signature = await Crypto.sign("important data", privateKey, "rsa-sha256")

// Verificación de firma
let isValid = await Crypto.verifySignature("important data", signature, publicKey)
```

#### 5. **Key Management Module** (`Crypto.Key`)
**Funcionalidades:**
- Generación de claves seguras
- Import/Export en múltiples formatos
- Derivación de claves (PBKDF2, HKDF)

**API Design:**
```vela
// Generación de claves
let aesKey = await Crypto.Key.generate("symmetric", "aes256")
let rsaKeys = await Crypto.Key.generate("asymmetric", "rsa-2048")

// Import/Export
let exported = await Crypto.Key.export(privateKey, "pkcs8")
let imported = await Crypto.Key.import("pkcs8", exportedData)

// Derivación
let derivedKey = await Crypto.Key.derive(password, salt, {
    algorithm: "pbkdf2",
    iterations: 10000
})
```

### Decisiones Arquitectónicas

#### **API Unificada vs APIs Separadas**
- ✅ **Elegido**: API unificada bajo `Crypto.*`
- ❌ **Rechazado**: Múltiples imports separados
- **Razón**: Consistencia, discoverability, menor complejidad de uso

#### **Async por Defecto**
- ✅ **Elegido**: Todas las operaciones async
- ❌ **Rechazado**: Mix de sync/async
- **Razón**: Operaciones crypto pueden ser costosas, mejor UX

#### **Type Safety Estricto**
- ✅ **Elegido**: Enums para algoritmos, tipos específicos para keys
- ❌ **Rechazado**: Strings libres para algoritmos
- **Razón**: Previene errores en tiempo de compilación

#### **Seguridad por Defecto**
- ✅ **Elegido**: Algoritmos seguros predeterminados
- ❌ **Rechazado**: Configuración manual requerida
- **Razón**: Reduce errores de configuración de seguridad

### Consideraciones de Seguridad

#### **Algoritmos Recomendados**
- **Hashing**: Argon2 para passwords, SHA256 para datos
- **Encryption**: AES-256-GCM para symmetric
- **Signatures**: RSA-2048 o ECDSA P-256
- **JWT**: HS256 para internal, RS256 para external

#### **Key Management**
- Claves nunca expuestas en memoria más tiempo necesario
- Zeroing automático de claves sensibles
- Soporte para HSM/external key storage

#### **Random Generation**
- Uso exclusivo de `rand::thread_rng()` o HSM
- Nunca `rand::random()` para crypto

### Implementación Técnica

#### **Dependencies Planeadas**
```toml
# Hashing
sha2 = "0.10"           # SHA256/512
bcrypt = "0.15"         # Password hashing
argon2 = "0.5"          # Modern KDF

# Encryption
aes-gcm = "0.10"        # AES-GCM
chacha20poly1305 = "0.10" # ChaCha20

# JWT
jsonwebtoken = "9.0"    # JWT handling

# Signatures
rsa = "0.9"             # RSA signatures
ecdsa = "0.16"          # ECDSA signatures
p256 = "0.13"           # P-256 curve
p384 = "0.13"           # P-384 curve
p521 = "0.13"           # P-521 curve

# Utilities
rand = "0.8"            # CSPRNG
base64 = "0.21"         # Base64 encoding
hex = "0.4"             # Hex encoding
```

#### **Error Handling**
```vela
enum CryptoError {
    InvalidKey("Key is not valid for this operation"),
    InvalidAlgorithm("Algorithm not supported"),
    DecryptionFailed("Decryption failed - invalid key or data"),
    SignatureInvalid("Signature verification failed"),
    JWTExpired("JWT token has expired"),
    JWTInvalid("JWT token is malformed"),
    HashVerificationFailed("Hash verification failed")
}
```

### Testing Strategy

#### **Unit Tests**
- Tests para cada algoritmo individual
- Tests de edge cases (empty data, invalid keys)
- Tests de error handling

#### **Integration Tests**
- Tests end-to-end de flujos completos
- Tests de performance
- Tests de concurrencia

#### **Security Tests**
- Tests de timing attacks
- Tests de key leakage
- Tests de algorithm agility

## ✅ Criterios de Aceptación
- [x] Arquitectura de API diseñada y documentada
- [x] Módulos principales definidos (Hash, Encrypt, JWT, Sign, Key)
- [x] APIs type-safe especificadas
- [x] Algoritmos seguros seleccionados
- [x] Consideraciones de seguridad documentadas
- [x] Estrategia de testing definida
- [x] ADR creado en `docs/architecture/`
- [x] Documentación de tarea completada

## 🔗 Referencias
- **Jira:** [TASK-113BD](https://velalang.atlassian.net/browse/TASK-113BD)
- **Historia:** [VELA-605](https://velalang.atlassian.net/browse/VELA-605)
- **ADR:** `docs/architecture/ADR-113BD-crypto-api-architecture.md`