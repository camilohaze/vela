# TASK-000K: Diseñar Arquitectura del Package Manager

## 📋 Información General
- **Historia:** VELA-562 (Tooling Design - Phase 0)
- **Epic:** EPIC-00C: Tooling Design
- **Sprint:** 2
- **Estado:** Completado ✅
- **Prioridad:** P0 (Crítica)
- **Estimación:** 64 horas
- **Dependencias:** TASK-000J (CLI Architecture)

---

## 🎯 Objetivo

Diseñar la arquitectura completa del **Vela Package Manager**, incluyendo:

- **Registry architecture** (storage, metadata, API)
- **Dependency resolution** (algoritmo PubGrub)
- **Lockfile format** (reproducibilidad)
- **Security** (checksums, signatures)
- **Package hosting** (CDN, escalabilidad)

---

## 🏗️ Package Manager Architecture

### 1. Registry Architecture

#### 1.1 Estructura del Registry

```
vela-registry.io/
├── /api/v1/                    # REST API
│   ├── /packages/              # List packages
│   ├── /packages/<name>        # Package info
│   ├── /packages/<name>/<ver>  # Version info
│   ├── /search?q=<query>       # Search
│   └── /publish                # Publish new version
│
├── /packages/                  # Package metadata (JSON)
│   └── <name>/
│       ├── versions.json       # All versions
│       └── <version>/
│           ├── metadata.json   # Package metadata
│           └── package.tar.gz  # Tarball
│
└── /static/                    # Static assets (docs, etc.)
```

---

#### 1.2 API Endpoints

##### `GET /api/v1/packages`
**Propósito:** Listar todos los packages

**Response:**
```json
{
  "packages": [
    {
      "name": "http",
      "latest_version": "2.0.0",
      "description": "HTTP client library",
      "downloads": 1250000
    },
    {
      "name": "json",
      "latest_version": "1.5.3",
      "description": "JSON parser and serializer",
      "downloads": 2100000
    }
  ],
  "total": 1523,
  "page": 1,
  "per_page": 50
}
```

---

##### `GET /api/v1/packages/<name>`
**Propósito:** Información del package

**Response:**
```json
{
  "name": "http",
  "description": "HTTP client and server library",
  "latest_version": "2.0.0",
  "homepage": "https://github.com/vela-lang/http",
  "repository": "https://github.com/vela-lang/http",
  "license": "MIT",
  "authors": ["Vela Team <team@velalang.io>"],
  "keywords": ["http", "client", "server", "rest"],
  "versions": ["2.0.0", "1.9.5", "1.9.4", "1.9.3"],
  "downloads": {
    "total": 1250000,
    "last_30_days": 45000
  },
  "created_at": "2024-01-15T10:30:00Z",
  "updated_at": "2025-11-20T14:25:00Z"
}
```

---

##### `GET /api/v1/packages/<name>/<version>`
**Propósito:** Información de versión específica

**Response:**
```json
{
  "name": "http",
  "version": "2.0.0",
  "description": "HTTP client and server library",
  "license": "MIT",
  "authors": ["Vela Team"],
  "dependencies": {
    "url": "^1.0.0",
    "tls": "^2.1.0"
  },
  "dev_dependencies": {
    "test-framework": "^3.0.0"
  },
  "checksum": "sha256:a3f2b1c8d9e7f6a5b4c3d2e1f0a9b8c7d6e5f4a3b2c1d0e9f8a7b6c5d4e3f2a1",
  "tarball": "https://vela-registry.io/packages/http/2.0.0/package.tar.gz",
  "size": 125456,
  "published_at": "2025-11-20T14:25:00Z",
  "published_by": "vela-team"
}
```

---

##### `GET /api/v1/search?q=<query>`
**Propósito:** Buscar packages

**Query Parameters:**
- `q`: Query string
- `page`: Página (default: 1)
- `per_page`: Resultados por página (default: 20, max: 100)
- `sort`: Sort order (relevance | downloads | recent)

**Response:**
```json
{
  "query": "http",
  "results": [
    {
      "name": "http",
      "version": "2.0.0",
      "description": "HTTP client and server library",
      "downloads": 1250000,
      "relevance": 0.95
    },
    {
      "name": "http-client",
      "version": "1.2.0",
      "description": "Lightweight HTTP client",
      "downloads": 85000,
      "relevance": 0.78
    }
  ],
  "total": 12,
  "page": 1,
  "per_page": 20
}
```

**Search algorithm:**
- Full-text search en name, description, keywords
- Ranking: TF-IDF + downloads weight
- Typo tolerance (Levenshtein distance)

---

##### `POST /api/v1/publish`
**Propósito:** Publicar nueva versión

**Headers:**
```
Authorization: Bearer <token>
Content-Type: multipart/form-data
```

**Body:**
```
package.tar.gz: <binary data>
metadata.json: { ... }
signature: <digital signature>
```

**Response:**
```json
{
  "success": true,
  "name": "my-package",
  "version": "1.0.0",
  "url": "https://vela-registry.io/packages/my-package/1.0.0"
}
```

**Validaciones:**
- Package name disponible
- Version no publicada previamente
- Signature válida
- Tarball válido (< 50 MB)
- Metadata válido

---

### 2. Package Metadata (`vela.yaml`)

#### 2.1 Formato Completo

```yaml
# REQUIRED FIELDS
name: http                       # Package name (lowercase, hyphens)
version: 2.0.0                   # SemVer version
authors:
  - "Vela Team <team@velalang.io>"

# OPTIONAL FIELDS
description: "HTTP client and server library"
license: MIT                     # SPDX identifier
homepage: "https://velalang.io/packages/http"
repository: "https://github.com/vela-lang/http"
documentation: "https://docs.velalang.io/http"
readme: "README.md"
keywords:
  - http
  - client
  - server
  - rest
categories:
  - networking
  - web

# DEPENDENCIES
dependencies:
  url: "^1.0.0"                 # Caret range
  tls: ">=2.1.0 <3.0.0"         # Explicit range
  json: "*"                      # Any version (not recommended)
  
dev-dependencies:
  test-framework: "^3.0.0"
  benchmark-suite: "^1.5.0"

# BUILD CONFIGURATION
build:
  exclude:                       # Files to exclude from tarball
    - "*.test.vela"
    - "docs/"
    - ".git/"
  include:                       # Files to force include
    - "LICENSE"
    - "README.md"

# PACKAGE METADATA
package:
  publish: true                  # Allow publishing to registry
  min-vela-version: "1.0.0"     # Minimum Vela version required
```

---

#### 2.2 SemVer Version Ranges

| Range | Meaning | Example matches |
|-------|---------|-----------------|
| `^1.2.3` | >= 1.2.3, < 2.0.0 | 1.2.3, 1.3.0, 1.9.9 |
| `~1.2.3` | >= 1.2.3, < 1.3.0 | 1.2.3, 1.2.9 |
| `1.2.*` | >= 1.2.0, < 1.3.0 | 1.2.0, 1.2.5 |
| `>=1.2.0 <2.0.0` | Explicit range | 1.2.0, 1.9.9 |
| `*` | Any version | 0.1.0, 5.2.1 |

**Reglas:**
- `^` es el default (recommended)
- `*` debe usarse con cuidado (no reproducible)

---

### 3. Dependency Resolution Algorithm

#### 3.1 PubGrub Algorithm

**Elección:** PubGrub (usado por Dart pub, Swift Package Manager)

**Ventajas:**
- ✅ Encuentra solución óptima (si existe)
- ✅ Mensajes de error claros (explica por qué falla)
- ✅ Maneja conflictos complejos
- ✅ Más eficiente que backtracking naïve

**Algoritmo (simplificado):**
```
1. Partial solution: Set de packages con versiones
2. Invariante: Partial solution es compatible

Loop:
  3. Unit propagation: derivar hechos del partial solution
  4. Si partial solution completo → SUCCESS
  5. Si contradicción → CONFLICT (backtrack o error)
  6. Decision: elegir package/version para agregar
  7. Agregar a partial solution
  8. Goto 3
```

---

#### 3.2 Ejemplo de Resolución

**Dependencias:**
```yaml
# my-project/vela.yaml
dependencies:
  http: "^2.0.0"
  json: "^1.5.0"

# http v2.0.0 depends on:
dependencies:
  url: "^1.0.0"
  tls: "^2.1.0"

# json v1.5.0 depends on:
dependencies:
  string-utils: "^1.2.0"
```

**Resolución:**
```
1. Root: my-project v0.1.0
2. http ^2.0.0 → http v2.0.0 (latest)
3. json ^1.5.0 → json v1.5.3 (latest)
4. url ^1.0.0 (from http) → url v1.2.1
5. tls ^2.1.0 (from http) → tls v2.3.0
6. string-utils ^1.2.0 (from json) → string-utils v1.4.2

Solution:
  http: 2.0.0
  json: 1.5.3
  url: 1.2.1
  tls: 2.3.0
  string-utils: 1.4.2
```

---

#### 3.3 Manejo de Conflictos

**Caso: Incompatibilidad de versiones**

```yaml
# my-project depends on:
dependencies:
  lib-a: "^1.0.0"
  lib-b: "^2.0.0"

# lib-a v1.0.0 depends on:
dependencies:
  shared: "^1.0.0"

# lib-b v2.0.0 depends on:
dependencies:
  shared: "^2.0.0"
```

**Error message:**
```
error: failed to resolve dependencies

Because lib-a 1.0.0 depends on shared ^1.0.0
  and lib-b 2.0.0 depends on shared ^2.0.0,
  lib-a 1.0.0 is incompatible with lib-b 2.0.0.

And because my-project depends on lib-a ^1.0.0 and lib-b ^2.0.0,
  version solving failed.

Suggestions:
  - Update lib-a to a version compatible with shared ^2.0.0
  - Update lib-b to a version compatible with shared ^1.0.0
  - Use different versions of lib-a or lib-b
```

---

### 4. Lockfile Format (`vela.lock`)

#### 4.1 Propósito

**Garantizar builds reproducibles:**
- Mismo código + mismo vela.lock → mismas dependencias
- CI/CD usa versiones exactas
- Evita "works on my machine"

---

#### 4.2 Formato

```toml
# vela.lock
# This file is generated by 'vela build'. Do not edit manually.

version = 1

[[packages]]
name = "http"
version = "2.0.0"
source = "registry+https://vela-registry.io/"
checksum = "sha256:a3f2b1c8d9e7f6a5b4c3d2e1f0a9b8c7d6e5f4a3b2c1d0e9f8a7b6c5d4e3f2a1"
dependencies = ["url ^1.0.0", "tls ^2.1.0"]

[[packages]]
name = "json"
version = "1.5.3"
source = "registry+https://vela-registry.io/"
checksum = "sha256:b4c3d2e1f0a9b8c7d6e5f4a3b2c1d0e9f8a7b6c5d4e3f2a1a3f2b1c8d9e7f6a5"
dependencies = ["string-utils ^1.2.0"]

[[packages]]
name = "url"
version = "1.2.1"
source = "registry+https://vela-registry.io/"
checksum = "sha256:c5d4e3f2a1a3f2b1c8d9e7f6a5b4c3d2e1f0a9b8c7d6e5f4a3b2c1d0e9f8a7b6"
dependencies = []

[[packages]]
name = "tls"
version = "2.3.0"
source = "registry+https://vela-registry.io/"
checksum = "sha256:d6e5f4a3b2c1d0e9f8a7b6c5d4e3f2a1a3f2b1c8d9e7f6a5b4c3d2e1f0a9b8c7"
dependencies = []

[[packages]]
name = "string-utils"
version = "1.4.2"
source = "registry+https://vela-registry.io/"
checksum = "sha256:e7f6a5b4c3d2e1f0a9b8c7d6e5f4a3b2c1d0e9f8a7b6c5d4e3f2a1a3f2b1c8d9"
dependencies = []
```

---

#### 4.3 Update Strategy

**Comando `vela update`:**
```bash
# Update all dependencies (respecting ranges in vela.yaml)
$ vela update
    Updating http v2.0.0 -> v2.1.0
    Updating json v1.5.3 -> v1.5.4
    Updated vela.lock

# Update specific package
$ vela update http
    Updating http v2.0.0 -> v2.1.0
    Updated vela.lock
```

**Lockfile diffs:**
```diff
 [[packages]]
 name = "http"
-version = "2.0.0"
+version = "2.1.0"
 source = "registry+https://vela-registry.io/"
-checksum = "sha256:a3f2b1c8..."
+checksum = "sha256:f7e8d9c2..."
```

---

### 5. Security

#### 5.1 Checksums (SHA-256)

**Generación al publicar:**
```bash
$ sha256sum package.tar.gz
a3f2b1c8d9e7f6a5b4c3d2e1f0a9b8c7d6e5f4a3b2c1d0e9f8a7b6c5d4e3f2a1  package.tar.gz
```

**Verificación al descargar:**
```bash
$ vela add http
  Downloading http v2.0.0...
  Verifying checksum... ✔
  Extracting...
```

**Error si checksum no coincide:**
```
error: checksum mismatch for http v2.0.0
  expected: sha256:a3f2b1c8...
  got:      sha256:f7e8d9c2...
  
This indicates the package may be corrupted or tampered with.
```

---

#### 5.2 Digital Signatures (Ed25519)

**Signing al publicar:**
```bash
# Generate keypair (one-time)
$ vela auth generate-key
Generated keypair:
  Public key:  ed25519:Abc123...
  Private key: ~/.vela/keys/private.key (keep secret!)

# Sign package
$ vela publish
Signing package with key ed25519:Abc123...
✔ Signature: ed25519:sig_Xyz789...
✔ Published my-package v1.0.0
```

**Verificación al descargar:**
```bash
$ vela add my-package
  Downloading my-package v1.0.0...
  Verifying signature... ✔
  Signed by: vela-team (ed25519:Abc123...)
```

**Trust model:**
- Registry almacena public key de cada publisher
- Packages firmados con private key del publisher
- CLI verifica firma con public key

---

#### 5.3 Vulnerability Scanning

**Comando `vela audit`:**
```bash
$ vela audit
Auditing dependencies for known vulnerabilities...

┌───────────────────┬─────────────────────────────────────┐
│ Package           │ Vulnerability                       │
├───────────────────┼─────────────────────────────────────┤
│ http@1.9.0        │ CVE-2024-12345: DoS vulnerability   │
│ Severity: HIGH    │ Patched in: 2.0.0                  │
│ More info: https://nvd.nist.gov/vuln/detail/CVE-2024-12345│
└───────────────────┴─────────────────────────────────────┘

Found 1 vulnerability. Run 'vela update' to fix.

$ vela update http
    Updating http v1.9.0 -> v2.0.0 (fixes CVE-2024-12345)
```

**Database:**
- Vela Security Advisory Database (vela-security.io)
- Sincronizado con NVD (National Vulnerability Database)
- Community-contributed advisories

---

### 6. Package Hosting

#### 6.1 Storage Backend

**AWS S3 (primary storage):**
```
s3://vela-packages/
├── packages/
│   └── <name>/
│       └── <version>/
│           └── package.tar.gz
└── metadata/
    └── <name>/
        ├── versions.json
        └── <version>/
            └── metadata.json
```

**Ventajas:**
- ✅ Durable (99.999999999% durability)
- ✅ Escalable (infinito)
- ✅ Económico ($0.023/GB/mes)
- ✅ Versionado automático

---

#### 6.2 CDN (CloudFront)

**Distribución global:**
```
CloudFront distribution:
  Origin: s3://vela-packages/
  Edge locations: 450+ worldwide
  Cache TTL: 1 year (immutable packages)
  
URLs:
  https://cdn.vela-registry.io/packages/http/2.0.0/package.tar.gz
```

**Performance:**
- Download speed: 10-50 MB/s (según región)
- Latency: < 50ms (desde edge locations)
- Cache hit ratio: > 95% (packages son immutable)

---

#### 6.3 Metadata Database

**PostgreSQL:**
```sql
-- Schema simplificado

CREATE TABLE packages (
  id SERIAL PRIMARY KEY,
  name VARCHAR(255) UNIQUE NOT NULL,
  description TEXT,
  homepage VARCHAR(500),
  repository VARCHAR(500),
  license VARCHAR(50),
  created_at TIMESTAMP DEFAULT NOW(),
  updated_at TIMESTAMP DEFAULT NOW()
);

CREATE TABLE package_versions (
  id SERIAL PRIMARY KEY,
  package_id INTEGER REFERENCES packages(id),
  version VARCHAR(50) NOT NULL,
  checksum VARCHAR(64) NOT NULL,
  signature VARCHAR(256),
  tarball_url VARCHAR(500),
  size_bytes INTEGER,
  published_at TIMESTAMP DEFAULT NOW(),
  published_by VARCHAR(255),
  downloads INTEGER DEFAULT 0,
  UNIQUE(package_id, version)
);

CREATE TABLE dependencies (
  id SERIAL PRIMARY KEY,
  package_version_id INTEGER REFERENCES package_versions(id),
  dependency_name VARCHAR(255) NOT NULL,
  version_range VARCHAR(100) NOT NULL,
  is_dev BOOLEAN DEFAULT FALSE
);

CREATE INDEX idx_packages_name ON packages(name);
CREATE INDEX idx_versions_package ON package_versions(package_id);
CREATE INDEX idx_deps_version ON dependencies(package_version_id);
```

**Queries:**
- `GET /packages/<name>`: SELECT con JOIN (< 5ms)
- `GET /search?q=`: Full-text search con pg_trgm (< 50ms)
- Downloads tracking: UPDATE async (no blocking)

---

#### 6.4 Cache Layer (Redis)

**Caching:**
```
Redis keys:
  package:<name>              → Package metadata (TTL: 1h)
  package:<name>:<version>    → Version metadata (TTL: 24h)
  search:<query>:<page>       → Search results (TTL: 15min)
  stats:downloads             → Download counters (persistent)
```

**Performance:**
- Cache hit ratio: > 80%
- Query latency: < 1ms (cache hit), < 10ms (cache miss)

---

### 7. Escalabilidad

#### 7.1 Capacity Planning

**Estimaciones (5 años):**
- Packages: 50,000
- Versions: 500,000 (10 versions/package promedio)
- Total storage: 50 TB
- Monthly downloads: 100M
- API requests: 1B/mes

**Costos estimados (AWS):**
```
S3 storage:        50 TB × $0.023/GB = $1,150/mes
CloudFront:        100M downloads × $0.085/GB = $8,500/mes
RDS PostgreSQL:    db.r6g.large = $300/mes
ElastiCache Redis: cache.r6g.large = $200/mes
──────────────────────────────────────────────────
Total:             ~$10,150/mes
```

**Crecimiento esperado:**
- Año 1: 5,000 packages
- Año 2: 15,000 packages
- Año 3: 30,000 packages
- Año 5: 50,000+ packages

---

#### 7.2 Rate Limiting

**Limits per API key:**
```
Anonymous:
  - 100 req/min (search, metadata)
  - 10 downloads/min

Authenticated:
  - 1,000 req/min
  - 100 downloads/min
  
Publishers:
  - 10,000 req/min
  - 10 publishes/hour
```

**Headers:**
```
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 87
X-RateLimit-Reset: 1701234567
```

---

### 8. Comparación con Otros Package Managers

| Feature | Vela PM | npm | Cargo | Go Modules |
|---------|---------|-----|-------|------------|
| **Algorithm** | PubGrub | SAT solver | SAT solver | MVS |
| **Lockfile** | ✅ vela.lock | ✅ package-lock.json | ✅ Cargo.lock | ✅ go.sum |
| **Security** | Checksums + Sigs | Checksums | Checksums | Checksums |
| **CDN** | CloudFront | CloudFlare | CloudFront | GCP CDN |
| **Audit** | ✅ vela audit | ✅ npm audit | ✅ cargo audit | ✅ govulncheck |
| **Monorepo** | ⏳ Planned | ✅ Workspaces | ✅ Workspaces | ✅ Modules |

---

## ✅ Criterios de Aceptación

- [x] Registry architecture definida (S3 + CloudFront + PostgreSQL + Redis)
- [x] API endpoints especificados (REST API v1)
- [x] Dependency resolution algorithm seleccionado (PubGrub)
- [x] Lockfile format diseñado (vela.lock)
- [x] Security mechanisms especificados (checksums SHA-256 + signatures Ed25519)
- [x] Package hosting strategy definida (AWS con CDN)
- [x] Escalabilidad planificada (50K packages, 100M downloads/mes)
- [x] Comparación con npm, Cargo, Go Modules

---

## 🔗 Referencias

### Algorithms
- [PubGrub Algorithm (Dart)](https://github.com/dart-lang/pub/blob/master/doc/solver.md)
- [Version SAT Solving (npm)](https://github.com/npm/cli/blob/latest/workspaces/arborist/README.md)
- [Minimal Version Selection (Go)](https://research.swtch.com/vgo-mvs)

### Implementations
- [npm Registry](https://docs.npmjs.com/cli/v10/using-npm/registry)
- [Cargo Registry (crates.io)](https://doc.rust-lang.org/cargo/reference/registry-index.html)
- [Go Module Proxy](https://go.dev/ref/mod#module-proxy)

### Security
- [SLSA Framework](https://slsa.dev/)
- [Sigstore](https://www.sigstore.dev/)
- [NVD (National Vulnerability Database)](https://nvd.nist.gov/)

---

**Estado:** ✅ Diseño completo  
**Prioridad:** P0 - Crítico para ecosystem growth  
**Siguiente paso:** TASK-000L (LSP Architecture)
