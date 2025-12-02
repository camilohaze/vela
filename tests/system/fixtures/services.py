"""
System Test Fixtures - Servicios Mock

Servicios mock realistas para tests de sistema DI y REST.
Estos servicios simulan componentes reales de una aplicación.

Jira: VELA-575, TASK-035J
"""

import random
from typing import Dict, List, Optional
from src.runtime.di import injectable, Scope


@injectable(scope=Scope.SINGLETON)
class DatabaseConnection:
    """
    Mock de conexión a base de datos (SINGLETON).
    
    En tests de sistema, simula una DB real con:
    - Estado compartido (queries_executed)
    - Lifecycle (connected/disconnected)
    - Query execution tracking
    """
    
    def __init__(self):
        self.connected = True
        self.queries_executed: List[str] = []
        self.connection_id = random.randint(1, 1000000)
    
    def execute(self, query: str) -> Dict:
        """Ejecutar query (mock)."""
        if not self.connected:
            raise RuntimeError("Database not connected")
        
        self.queries_executed.append(query)
        return {
            "success": True,
            "query": query,
            "connection_id": self.connection_id
        }
    
    def disconnect(self):
        """Desconectar (para tests de cleanup)."""
        self.connected = False
        self.queries_executed.clear()
    
    def reset(self):
        """Reset state (para tests)."""
        self.queries_executed.clear()


@injectable
class UserRepository:
    """
    Mock de repositorio de usuarios (TRANSIENT por defecto).
    
    Depende de DatabaseConnection.
    """
    
    def __init__(self, db: DatabaseConnection):
        self.db = db
        self._users: Dict[int, Dict] = {}
        self._next_id = 1
    
    def find_by_id(self, user_id: int) -> Optional[Dict]:
        """Buscar usuario por ID."""
        query = f"SELECT * FROM users WHERE id = {user_id}"
        self.db.execute(query)
        return self._users.get(user_id)
    
    def find_all(self) -> List[Dict]:
        """Listar todos los usuarios."""
        self.db.execute("SELECT * FROM users")
        return list(self._users.values())
    
    def create(self, data: Dict) -> Dict:
        """Crear usuario."""
        user_id = self._next_id
        self._next_id += 1
        
        user = {
            "id": user_id,
            **data
        }
        
        self._users[user_id] = user
        self.db.execute(f"INSERT INTO users VALUES ({user})")
        
        return user
    
    def update(self, user_id: int, data: Dict) -> Optional[Dict]:
        """Actualizar usuario."""
        if user_id not in self._users:
            return None
        
        self._users[user_id].update(data)
        self.db.execute(f"UPDATE users SET {data} WHERE id = {user_id}")
        
        return self._users[user_id]
    
    def delete(self, user_id: int) -> bool:
        """Eliminar usuario."""
        if user_id not in self._users:
            return False
        
        del self._users[user_id]
        self.db.execute(f"DELETE FROM users WHERE id = {user_id}")
        
        return True


@injectable
class UserService:
    """
    Mock de servicio de usuarios (TRANSIENT por defecto).
    
    Capa de lógica de negocio sobre UserRepository.
    """
    
    def __init__(self, repository: UserRepository):
        self.repository = repository
    
    def get_user(self, user_id: int) -> Optional[Dict]:
        """Obtener usuario por ID."""
        user = self.repository.find_by_id(user_id)
        
        if user:
            # Agregar campo computado
            user["display_name"] = f"{user.get('name', 'Unknown')} (ID: {user['id']})"
        
        return user
    
    def list_users(self) -> List[Dict]:
        """Listar todos los usuarios."""
        return self.repository.find_all()
    
    def create_user(self, name: str, email: str) -> Dict:
        """Crear nuevo usuario."""
        if not name or not email:
            raise ValueError("Name and email are required")
        
        if "@" not in email:
            raise ValueError("Invalid email format")
        
        return self.repository.create({
            "name": name,
            "email": email
        })
    
    def update_user(self, user_id: int, name: Optional[str] = None, email: Optional[str] = None) -> Optional[Dict]:
        """Actualizar usuario existente."""
        data = {}
        if name:
            data["name"] = name
        if email:
            if "@" not in email:
                raise ValueError("Invalid email format")
            data["email"] = email
        
        if not data:
            raise ValueError("No data to update")
        
        return self.repository.update(user_id, data)
    
    def delete_user(self, user_id: int) -> bool:
        """Eliminar usuario."""
        return self.repository.delete(user_id)


@injectable
class AuthService:
    """
    Mock de servicio de autenticación (TRANSIENT por defecto).
    
    Simula login/logout con tokens JWT.
    """
    
    def __init__(self, user_repo: UserRepository):
        self.user_repo = user_repo
        self._tokens: Dict[str, int] = {}  # token -> user_id
    
    def login(self, email: str, password: str) -> Optional[Dict]:
        """
        Login de usuario.
        
        Mock simple: acepta cualquier password si el email existe.
        """
        # Buscar usuario por email (en mock, buscar en _users)
        users = self.user_repo.find_all()
        user = next((u for u in users if u.get("email") == email), None)
        
        if not user:
            return None
        
        # Generar token
        token = f"jwt-token-{random.randint(1000, 9999)}"
        self._tokens[token] = user["id"]
        
        return {
            "token": token,
            "user": user
        }
    
    def logout(self, token: str) -> bool:
        """Logout (invalidar token)."""
        if token in self._tokens:
            del self._tokens[token]
            return True
        return False
    
    def validate_token(self, token: str) -> Optional[int]:
        """Validar token y retornar user_id."""
        return self._tokens.get(token)
    
    def get_current_user(self, token: str) -> Optional[Dict]:
        """Obtener usuario actual por token."""
        user_id = self.validate_token(token)
        if not user_id:
            return None
        
        return self.user_repo.find_by_id(user_id)


@injectable(scope=Scope.SCOPED)
class RequestContext:
    """
    Mock de contexto de request HTTP (SCOPED).
    
    Vive solo durante una request HTTP.
    Útil para tests de SCOPED behavior.
    """
    
    def __init__(self):
        self.request_id = random.randint(1, 1000000)
        self.metadata: Dict = {}
        self.start_time = None
    
    def set_metadata(self, key: str, value):
        """Agregar metadata al contexto."""
        self.metadata[key] = value
    
    def get_metadata(self, key: str):
        """Obtener metadata."""
        return self.metadata.get(key)


@injectable
class Logger:
    """
    Mock de logger (TRANSIENT por defecto).
    
    Para tests de lifecycle hooks.
    """
    
    def __init__(self, context: Optional[RequestContext] = None):
        self.context = context
        self.logs: List[str] = []
    
    def log(self, message: str):
        """Log mensaje."""
        prefix = f"[Request {self.context.request_id}] " if self.context else ""
        log_entry = f"{prefix}{message}"
        self.logs.append(log_entry)
        print(log_entry)
    
    def info(self, message: str):
        """Log nivel INFO."""
        self.log(f"INFO: {message}")
    
    def error(self, message: str):
        """Log nivel ERROR."""
        self.log(f"ERROR: {message}")


@injectable
class CacheService:
    """
    Mock de servicio de caché (TRANSIENT por defecto).
    
    Para tests de multi providers.
    """
    
    def __init__(self):
        self._cache: Dict[str, any] = {}
    
    def get(self, key: str) -> Optional[any]:
        """Obtener valor del caché."""
        return self._cache.get(key)
    
    def set(self, key: str, value: any):
        """Guardar valor en caché."""
        self._cache[key] = value
    
    def delete(self, key: str) -> bool:
        """Eliminar valor del caché."""
        if key in self._cache:
            del self._cache[key]
            return True
        return False
    
    def clear(self):
        """Limpiar todo el caché."""
        self._cache.clear()


# Factory provider examples para tests
def create_database_connection() -> DatabaseConnection:
    """Factory para DatabaseConnection."""
    db = DatabaseConnection()
    db.execute("SELECT 1")  # Warmup query
    return db


def create_logger_with_context(context: RequestContext) -> Logger:
    """Factory para Logger con contexto."""
    logger = Logger(context)
    logger.log("Logger initialized")
    return logger


# Classes con lifecycle hooks para tests
@injectable
class ServiceWithLifecycle:
    """
    Servicio con lifecycle hooks (OnInit, OnDestroy).
    
    Para tests de lifecycle.
    """
    
    def __init__(self, db: DatabaseConnection):
        self.db = db
        self.initialized = False
        self.destroyed = False
    
    def on_init(self):
        """Hook: OnInit."""
        self.initialized = True
        self.db.execute("INIT ServiceWithLifecycle")
    
    def on_destroy(self):
        """Hook: OnDestroy."""
        self.destroyed = True
        self.db.execute("DESTROY ServiceWithLifecycle")
