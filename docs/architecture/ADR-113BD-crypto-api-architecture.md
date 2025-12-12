# ADR-113BD: Arquitectura de Crypto API

## Estado
✅ Aceptado

## Fecha
2025-12-11

## Contexto
Como parte de la historia US-24J, necesitamos implementar una API de criptografía estándar que proporcione funcionalidades seguras para hashing, encriptación, firmas digitales y JWT. Esta API debe ser:

- **Type-safe**: Validación de tipos en tiempo de compilación
- **Fácil de usar**: API intuitiva y consistente
- **Segura por defecto**: Algoritmos seguros con configuraciones predeterminadas
- **Extensible**: Soporte para algoritmos adicionales
- **Performante**: Implementaciones optimizadas

## Decisión
Implementaremos una Crypto API unificada con los siguientes módulos:

### 1. Hashing Module
```vela
module Crypto.Hash {
    // Algoritmos soportados
    enum HashAlgorithm {
        SHA256,
        SHA512,
        Bcrypt { cost: Number },
        Argon2 { iterations: Number, memory: Number, parallelism: Number }
    }

    // API principal
    async fn hash(data: String | Bytes, algorithm: HashAlgorithm) -> Result<String>
    async fn verify(data: String | Bytes, hash: String) -> Result<Bool>
}
```

### 2. Symmetric Encryption Module
```vela
module Crypto.Encrypt {
    // Algoritmos soportados
    enum EncryptionAlgorithm {
        AES256_GCM,
        ChaCha20_Poly1305
    }

    // API principal
    async fn generateKey(algorithm: EncryptionAlgorithm) -> Result<Key>
    async fn encrypt(data: Bytes, key: Key, options: EncryptOptions) -> Result<EncryptedData>
    async fn decrypt(encryptedData: EncryptedData, key: Key) -> Result<Bytes>
}
```

### 3. JWT Module
```vela
module Crypto.JWT {
    // Algoritmos soportados
    enum JWTAlgorithm {
        HS256, HS384, HS512,  // HMAC
        RS256, RS384, RS512,  // RSA
        ES256, ES384, ES512   // ECDSA
    }

    // API principal
    async fn create(payload: JsonObject, key: Key, options: JWTOptions) -> Result<String>
    async fn verify(token: String, key: Key) -> Result<JsonObject>
    async fn decode(token: String) -> Result<JWTDecoded>
}
```

### 4. Digital Signatures Module
```vela
module Crypto.Sign {
    // Algoritmos soportados
    enum SignatureAlgorithm {
        RSA_SHA256, RSA_SHA384, RSA_SHA512,
        ECDSA_SHA256, ECDSA_SHA384, ECDSA_SHA512
    }

    // API principal
    async fn generateKeyPair(algorithm: SignatureAlgorithm) -> Result<KeyPair>
    async fn sign(data: Bytes, privateKey: Key) -> Result<Signature>
    async fn verify(data: Bytes, signature: Signature, publicKey: Key) -> Result<Bool>
}
```

### 5. Key Management Module
```vela
module Crypto.Key {
    // Tipos de clave
    enum KeyType {
        Symmetric,
        Private,
        Public
    }

    // API principal
    async fn generate(type: KeyType, algorithm: String) -> Result<Key>
    async fn import(format: KeyFormat, data: Bytes) -> Result<Key>
    async fn export(key: Key, format: KeyFormat) -> Result<Bytes>
    async fn derive(password: String, salt: Bytes, options: KDFOptions) -> Result<Key>
}
```

## Consecuencias

### Positivas
- **API Unificada**: Una sola API para todas las operaciones criptográficas
- **Type Safety**: Validación de tipos en tiempo de compilación
- **Seguridad**: Algoritmos seguros con configuraciones predeterminadas
- **Performance**: Implementaciones optimizadas y asíncronas
- **Extensibilidad**: Fácil agregar nuevos algoritmos

### Negativas
- **Complejidad**: API más compleja que wrappers simples
- **Dependencias**: Requiere múltiples crates de Rust crypto
- **Tamaño**: Bundle más grande por incluir múltiples algoritmos

## Alternativas Consideradas

### 1. Múltiples APIs Separadas
```vela
// Alternativa: APIs separadas
import 'crypto:hash'
import 'crypto:encrypt'
import 'crypto:jwt'
import 'crypto:sign'
```

**Rechazada porque**: Mayor complejidad de importación y uso inconsistente.

### 2. Wrapper Simple de Rust Crates
```vela
// Alternativa: Wrappers directos
let hash = sha256("data")
let encrypted = aes256_encrypt(data, key)
```

**Rechazada porque**: No type-safe, APIs inconsistentes, difícil de mantener.

### 3. Configuración Manual de Algoritmos
```vela
// Alternativa: Configuración manual
let hasher = SHA256.new()
hasher.update(data)
let result = hasher.finalize()
```

**Rechazada porque**: Verboso, propenso a errores, no seguro por defecto.

## Implementación

### Estructura de Archivos
```
packages/crypto/
├── src/
│   ├── lib.rs           # Exports principales
│   ├── hash.rs          # Hashing implementations
│   ├── encrypt.rs       # Symmetric encryption
│   ├── jwt.rs           # JWT handling
│   ├── sign.rs          # Digital signatures
│   ├── key.rs           # Key management
│   └── error.rs         # Crypto errors
├── Cargo.toml           # Dependencies
└── tests/
    ├── hash_tests.rs
    ├── encrypt_tests.rs
    ├── jwt_tests.rs
    └── sign_tests.rs
```

### Dependencies
```toml
[dependencies]
# Hashing
sha2 = "0.10"
bcrypt = "0.15"
argon2 = "0.5"

# Encryption
aes-gcm = "0.10"
chacha20poly1305 = "0.10"

# JWT
jsonwebtoken = "9.0"

# Signatures
rsa = "0.9"
ecdsa = "0.16"
p256 = "0.13"
p384 = "0.13"
p521 = "0.13"

# Random
rand = "0.8"

# Async
tokio = { version = "1.0", features = ["full"] }
```

### Error Handling
```vela
enum CryptoError {
    InvalidKey,
    InvalidAlgorithm,
    DecryptionFailed,
    SignatureInvalid,
    JWTExpired,
    HashVerificationFailed
}
```

## Referencias
- Jira: [VELA-605](https://velalang.atlassian.net/browse/VELA-605)
- Historia: [US-24J](https://velalang.atlassian.net/browse/US-24J)
- RFC: [Crypto API Design](https://github.com/velalang/rfcs/pull/XXX)
- Standards: [NIST Cryptographic Standards](https://csrc.nist.gov/projects/cryptographic-standards-and-guidelines)