# Arquitectura del Package Manager de Vela

**Historia:** VELA-562 (US-00C)  
**Subtask:** TASK-000K  
**Fecha:** 2025-11-30  
**Estado:** ✅ Completado

---

## 📋 Resumen Ejecutivo

Este documento define la arquitectura completa del **Vela Package Manager**, el sistema centralizado para publicar, descubrir, instalar y gestionar paquetes del ecosistema Vela. El diseño se inspira en npm, Cargo y pub.dev, pero optimizado para las necesidades específicas de Vela.

---

## 1. Registry Architecture

### **1.1. Arquitectura Elegida: Hybrid (Centralized Metadata + Federated Storage)**

**Modelo:**
- ✅ **Metadata centralizado**: PostgreSQL en `registry.velalang.org`
- ✅ **Package storage federado**: CDN (CloudFront + S3) para binarios

**Razones:**
- ✅ **Performance**: CDN distribuye paquetes globalmente (baja latencia)
- ✅ **Reliability**: S3 con 99.999999999% durability
- ✅ **Scalability**: Metadata en PostgreSQL escala horizontalmente
- ✅ **Cost-effective**: Storage barato en S3, queries rápidas en PostgreSQL

**Alternativas descartadas:**
- ❌ **Fully centralized (como npm < 2018)**: Single point of failure, latencia alta
- ❌ **Fully federated (como IPFS)**: Complejo, lento para discovery
- ❌ **Blockchain-based**: Overhead innecesario, costos altos

---

### **1.2. Componentes del Registry**

```
┌────────────────────────────────────────────────────────────┐
│                   registry.velalang.org                     │
│  ┌─────────────────────────────────────────────────────┐   │
│  │              REST API (Rust/Actix-web)              │   │
│  │  - /api/packages          (search, list)            │   │
│  │  - /api/packages/:name    (metadata)                │   │
│  │  - /api/packages/:name/versions (versions)          │   │
│  │  - /api/publish           (publish package)         │   │
│  │  - /api/user/login        (authentication)          │   │
│  └──────────────────┬──────────────────────────────────┘   │
│                     │                                       │
│  ┌──────────────────v──────────────────────────────────┐   │
│  │           PostgreSQL (Metadata DB)                   │   │
│  │  - packages (name, description, owner, created_at)   │   │
│  │  - versions (package_id, version, checksum, size)    │   │
│  │  - users (email, api_token, created_at)              │   │
│  │  - downloads (package_id, version, count, date)      │   │
│  └─────────────────────────────────────────────────────┘   │
└────────────────────────────────────────────────────────────┘
                               │
                               │ (package downloads)
                               v
          ┌────────────────────────────────────────┐
          │      CDN (CloudFront + S3 Backend)     │
          │  packages.velalang.org                 │
          │  - /:package/:version/:file.tar.gz     │
          │  - Global edge locations                │
          └────────────────────────────────────────┘
```

---

## 2. Package Hosting

### **2.1. Storage: Amazon S3**

**Bucket structure:**
```
s3://vela-packages-prod/
├── http/
│   ├── 1.0.0/
│   │   ├── http-1.0.0.tar.gz        (source + metadata)
│   │   └── http-1.0.0.tar.gz.sha256 (checksum)
│   ├── 1.1.0/
│   │   ├── http-1.1.0.tar.gz
│   │   └── http-1.1.0.tar.gz.sha256
├── json/
│   ├── 2.3.1/
│   │   ├── json-2.3.1.tar.gz
│   │   └── json-2.3.1.tar.gz.sha256
```

**Access policy:**
- **Public read**: Todos pueden descargar
- **Authenticated write**: Solo propietarios pueden publicar
- **Immutable versions**: Una vez publicado, NO se puede sobrescribir

---

### **2.2. CDN: CloudFront**

**Configuración:**
- **Origin**: S3 bucket `vela-packages-prod`
- **Caching**: TTL 1 año (paquetes son inmutables)
- **Compression**: Gzip automático
- **Edge locations**: Global (50+ locations)

**URL format:**
```
https://packages.velalang.org/http/1.0.0/http-1.0.0.tar.gz
```

---

## 3. Metadata Storage (PostgreSQL)

### **3.1. Schema de Base de Datos**

#### **Tabla `packages`**
```sql
CREATE TABLE packages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) UNIQUE NOT NULL,  -- ej: "http"
    description TEXT,
    homepage_url VARCHAR(512),
    repository_url VARCHAR(512),
    license VARCHAR(100),  -- ej: "Apache-2.0"
    owner_id UUID REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW(),
    
    -- Full-text search
    tsv tsvector GENERATED ALWAYS AS (
        to_tsvector('english', name || ' ' || COALESCE(description, ''))
    ) STORED,
    
    INDEX idx_packages_name (name),
    INDEX idx_packages_tsv USING GIN (tsv)
);
```

#### **Tabla `versions`**
```sql
CREATE TABLE versions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    package_id UUID REFERENCES packages(id) ON DELETE CASCADE,
    version VARCHAR(20) NOT NULL,  -- ej: "1.2.3"
    description TEXT,
    
    -- Checksums
    checksum_sha256 VARCHAR(64) NOT NULL,  -- SHA-256 del .tar.gz
    size_bytes BIGINT NOT NULL,
    
    -- Dependencies
    dependencies JSONB DEFAULT '[]'::jsonb,  -- [{name, version, optional}]
    dev_dependencies JSONB DEFAULT '[]'::jsonb,
    
    -- Metadata
    published_at TIMESTAMP DEFAULT NOW(),
    yanked BOOLEAN DEFAULT FALSE,  -- Marcado como obsoleto
    
    -- S3 location
    storage_url VARCHAR(512) NOT NULL,  -- S3 path
    
    UNIQUE (package_id, version),
    INDEX idx_versions_package (package_id),
    INDEX idx_versions_published (published_at DESC)
);
```

#### **Tabla `users`**
```sql
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE NOT NULL,
    api_token VARCHAR(64) UNIQUE NOT NULL,  -- For `vela pkg publish`
    created_at TIMESTAMP DEFAULT NOW(),
    
    INDEX idx_users_email (email),
    INDEX idx_users_api_token (api_token)
);
```

#### **Tabla `downloads`**
```sql
CREATE TABLE downloads (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    package_id UUID REFERENCES packages(id) ON DELETE CASCADE,
    version_id UUID REFERENCES versions(id) ON DELETE CASCADE,
    count BIGINT DEFAULT 0,
    date DATE DEFAULT CURRENT_DATE,
    
    UNIQUE (package_id, version_id, date),
    INDEX idx_downloads_package (package_id),
    INDEX idx_downloads_date (date DESC)
);
```

---

### **3.2. Queries Comunes**

**Buscar paquetes:**
```sql
SELECT 
    p.name, 
    p.description, 
    COUNT(DISTINCT v.id) AS version_count,
    SUM(d.count) AS total_downloads
FROM packages p
LEFT JOIN versions v ON v.package_id = p.id
LEFT JOIN downloads d ON d.package_id = p.id
WHERE p.tsv @@ to_tsquery('english', 'json & parser')
GROUP BY p.id
ORDER BY total_downloads DESC
LIMIT 20;
```

**Obtener metadata de versión:**
```sql
SELECT 
    p.name,
    v.version,
    v.description,
    v.checksum_sha256,
    v.size_bytes,
    v.dependencies,
    v.storage_url
FROM versions v
JOIN packages p ON p.id = v.package_id
WHERE p.name = 'http' AND v.version = '1.2.3' AND v.yanked = FALSE;
```

---

## 4. `vela.yaml` Formal Schema

### **4.1. JSON Schema**

**Schema completo (JSON Schema draft-07):**
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Vela Package Manifest",
  "type": "object",
  "required": ["name", "version"],
  "properties": {
    "name": {
      "type": "string",
      "pattern": "^[a-z0-9_-]+$",
      "minLength": 1,
      "maxLength": 255,
      "description": "Package name (lowercase, alphanumeric, hyphens, underscores)"
    },
    "version": {
      "type": "string",
      "pattern": "^\\d+\\.\\d+\\.\\d+(-[a-z0-9.-]+)?(\\+[a-z0-9.-]+)?$",
      "description": "SemVer version (e.g., 1.2.3, 1.0.0-beta.1, 1.0.0+build.123)"
    },
    "description": {
      "type": "string",
      "maxLength": 500
    },
    "authors": {
      "type": "array",
      "items": {
        "type": "string",
        "pattern": ".+ <.+@.+>"
      },
      "description": "Authors in format 'Name <email@example.com>'"
    },
    "license": {
      "type": "string",
      "enum": ["MIT", "Apache-2.0", "GPL-3.0", "BSD-3-Clause", "Other"],
      "description": "SPDX license identifier"
    },
    "homepage": {
      "type": "string",
      "format": "uri"
    },
    "repository": {
      "type": "string",
      "format": "uri"
    },
    "dependencies": {
      "type": "object",
      "patternProperties": {
        "^[a-z0-9_-]+$": {
          "type": "string",
          "pattern": "^[~^>=<]?\\d+\\.\\d+\\.\\d+$",
          "description": "SemVer constraint (e.g., ^1.0.0, ~2.1.0, >=3.0.0)"
        }
      }
    },
    "dev_dependencies": {
      "type": "object",
      "patternProperties": {
        "^[a-z0-9_-]+$": {
          "type": "string"
        }
      }
    }
  }
}
```

**Ejemplo válido:**
```yaml
name: "http"
version: "1.2.3"
description: "HTTP client and server library"
authors:
  - "John Doe <john@example.com>"
license: "Apache-2.0"
homepage: "https://github.com/velalang/http"
repository: "https://github.com/velalang/http"
dependencies:
  json: "^2.0.0"
  crypto: "~1.5.0"
dev_dependencies:
  testing: "^0.5.0"
```

---

## 5. Dependency Resolution Algorithm

### **5.1. SemVer Rules**

**Vela sigue Semantic Versioning 2.0.0:**
- **Major** (1.x.x): Breaking changes
- **Minor** (x.1.x): New features (backward compatible)
- **Patch** (x.x.1): Bug fixes (backward compatible)

**Constraint operators:**
- `^1.2.3`: Compatible updates (>=1.2.3, <2.0.0)
- `~1.2.3`: Patch updates (>=1.2.3, <1.3.0)
- `>=1.2.3`: Greater or equal
- `<2.0.0`: Less than
- `1.2.3`: Exact version

---

### **5.2. PubGrub Algorithm**

**Elegido: PubGrub** (usado por Dart, Swift Package Manager).

**Razones:**
- ✅ **Correctness**: Garantiza resolver o reportar conflicto
- ✅ **Error messages**: Explica POR QUÉ no se puede resolver
- ✅ **Performance**: O(n²) en peor caso, O(n) promedio

**Alternativas descartadas:**
- ❌ **Backtracking (usado por npm < v7)**: Lento, mensajes de error confusos
- ❌ **SAT solver**: Overhead innecesario para problema relativamente simple

---

### **5.3. Algoritmo Simplificado**

```rust
fn resolve_dependencies(
    root: Package,
    registry: &Registry,
) -> Result<HashMap<String, Version>, ResolutionError> {
    let mut solution = HashMap::new();
    let mut to_visit = vec![(root.name, root.version)];
    
    while let Some((name, version)) = to_visit.pop() {
        // Check if already resolved
        if let Some(resolved_version) = solution.get(&name) {
            if !version.is_compatible_with(resolved_version) {
                return Err(ResolutionError::Conflict {
                    package: name,
                    required: version,
                    existing: resolved_version.clone(),
                });
            }
            continue;
        }
        
        // Fetch metadata
        let metadata = registry.get_metadata(&name, &version)?;
        solution.insert(name.clone(), version.clone());
        
        // Add dependencies to queue
        for (dep_name, constraint) in metadata.dependencies {
            let dep_version = registry.find_latest_matching(&dep_name, &constraint)?;
            to_visit.push((dep_name, dep_version));
        }
    }
    
    Ok(solution)
}
```

---

## 6. Lockfile Format (`vela.lock`)

### **6.1. Formato YAML**

**Propósito:**
- ✅ **Reproducibilidad**: Build idéntico en cualquier máquina
- ✅ **Integrity checks**: Detectar paquetes alterados

**Ejemplo de `vela.lock`:**
```yaml
# Generated by vela pkg install
# DO NOT EDIT THIS FILE MANUALLY

version: "1"

packages:
  - name: http
    version: 1.2.3
    checksum: sha256:a3b4c5d6e7f8...
    url: https://packages.velalang.org/http/1.2.3/http-1.2.3.tar.gz
    dependencies:
      - json@2.3.1
      - crypto@1.5.2
  
  - name: json
    version: 2.3.1
    checksum: sha256:1a2b3c4d5e6f...
    url: https://packages.velalang.org/json/2.3.1/json-2.3.1.tar.gz
    dependencies: []
  
  - name: crypto
    version: 1.5.2
    checksum: sha256:7f8e9d0c1b2a...
    url: https://packages.velalang.org/crypto/1.5.2/crypto-1.5.2.tar.gz
    dependencies: []
```

---

### **6.2. Lockfile Update Strategy**

**Reglas:**
1. `vela pkg install <pkg>`: Añadir al lockfile con versión exacta
2. `vela pkg update`: Actualizar a últimas versiones compatibles
3. `vela pkg update <pkg>`: Actualizar solo ese paquete

---

## 7. Package Publishing Workflow

### **7.1. Proceso de Publicación**

```bash
# 1. Crear cuenta (una vez)
vela pkg register

# 2. Login
vela pkg login
# Prompt: Email? john@example.com
# Prompt: API Token? (obtener de registry.velalang.org)

# 3. Validar vela.yaml
vela pkg validate

# 4. Publicar
vela pkg publish
```

**Flujo interno:**
1. CLI valida `vela.yaml` contra JSON Schema
2. CLI empaqueta source code en `.tar.gz`
3. CLI calcula SHA-256 checksum
4. CLI sube `.tar.gz` a S3 (via presigned URL)
5. CLI registra metadata en PostgreSQL via API
6. Registry valida y activa paquete

---

### **7.2. Validaciones Pre-Publicación**

**Checklist automático:**
- ✅ `vela.yaml` válido según schema
- ✅ Version no existe en registry
- ✅ Package name disponible (si es primera publicación)
- ✅ Tests pasan: `vela test`
- ✅ No vulnerabilidades conocidas en dependencies
- ✅ Licencia especificada
- ✅ README.md presente

---

## 8. REST API Endpoints

### **8.1. Endpoints Públicos**

#### **GET `/api/packages`** - Listar paquetes
**Query params:**
- `q`: Search query (full-text)
- `sort`: Ordenar por (`downloads`, `created`, `updated`)
- `limit`: Resultados por página (default: 20, max: 100)
- `offset`: Paginación

**Response:**
```json
{
  "packages": [
    {
      "name": "http",
      "description": "HTTP client and server",
      "latest_version": "1.2.3",
      "downloads": 12345,
      "created_at": "2025-01-15T10:00:00Z"
    }
  ],
  "total": 150,
  "limit": 20,
  "offset": 0
}
```

---

#### **GET `/api/packages/:name`** - Metadata de paquete
**Response:**
```json
{
  "name": "http",
  "description": "HTTP client and server library",
  "homepage": "https://github.com/velalang/http",
  "repository": "https://github.com/velalang/http",
  "license": "Apache-2.0",
  "versions": ["1.0.0", "1.1.0", "1.2.3"],
  "latest_version": "1.2.3",
  "downloads": 12345,
  "created_at": "2025-01-15T10:00:00Z"
}
```

---

#### **GET `/api/packages/:name/versions/:version`** - Metadata de versión
**Response:**
```json
{
  "name": "http",
  "version": "1.2.3",
  "description": "...",
  "checksum_sha256": "a3b4c5d6...",
  "size_bytes": 1024000,
  "dependencies": {
    "json": "^2.0.0",
    "crypto": "~1.5.0"
  },
  "dev_dependencies": {},
  "published_at": "2025-03-10T14:30:00Z",
  "download_url": "https://packages.velalang.org/http/1.2.3/http-1.2.3.tar.gz"
}
```

---

### **8.2. Endpoints Autenticados**

#### **POST `/api/publish`** - Publicar paquete
**Headers:**
- `Authorization: Bearer <api_token>`

**Body (multipart/form-data):**
- `metadata`: JSON con vela.yaml
- `package`: .tar.gz file

**Response:**
```json
{
  "success": true,
  "package": "http",
  "version": "1.2.3",
  "url": "https://packages.velalang.org/http/1.2.3/http-1.2.3.tar.gz"
}
```

---

#### **POST `/api/user/login`** - Autenticación
**Body:**
```json
{
  "email": "john@example.com",
  "api_token": "vela_..."
}
```

**Response:**
```json
{
  "success": true,
  "user": {
    "id": "uuid-...",
    "email": "john@example.com"
  }
}
```

---

## 9. Security & Trust

### **9.1. Checksums**

**SHA-256 para integrity:**
- ✅ Cada `.tar.gz` tiene checksum en metadata
- ✅ CLI valida checksum antes de instalar
- ✅ Si checksum no coincide → ERROR

---

### **9.2. HTTPS Only**

- ✅ Todos los endpoints usan HTTPS
- ✅ No fallback a HTTP

---

### **9.3. Rate Limiting**

**Límites por API key:**
- `/api/packages`: 1000 req/hour
- `/api/publish`: 10 req/hour
- `/api/user/login`: 10 req/hour

---

## 10. Monitoreo y Analytics

### **10.1. Métricas Clave**

- **Downloads por paquete/versión**
- **Tiempo de resolución de dependencias**
- **Errores de publicación**
- **Latencia de registry API**

---

### **10.2. Dashboard Público**

**URL:** `https://stats.velalang.org`

**Métricas públicas:**
- Total packages
- Total downloads
- Most popular packages
- Recent publishes

---

## 11. Testing

### **11.1. Tests Unitarios**

```rust
#[test]
fn test_semver_constraint_parsing() {
    let constraint = Constraint::parse("^1.2.3").unwrap();
    assert!(constraint.matches(&Version::new(1, 5, 0)));
    assert!(!constraint.matches(&Version::new(2, 0, 0)));
}
```

---

### **11.2. Tests de Integración**

**Escenarios:**
- Publicar paquete → Instalar → Validar checksum
- Resolver dependencias complejas (diamond dependency)
- Detectar conflictos irresolubles

---

## 12. Referencias

- **npm registry**: https://docs.npmjs.com/cli/v10/using-npm/registry
- **Cargo registry**: https://doc.rust-lang.org/cargo/reference/registry.html
- **PubGrub algorithm**: https://github.com/pubgrub-rs/pubgrub
- **SemVer spec**: https://semver.org/

---

**Autor:** Vela Core Team  
**Revisión:** 2025-11-30  
**Versión:** 1.0
