"""
File Upload Decorators - @file, @files, @form

Implementación de: TASK-035E
Historia: VELA-575
Fecha: 2025-12-01

Descripción:
Decoradores para manejo de file uploads y form data en endpoints HTTP:
- @file/@upload: Upload de archivo único (multipart/form-data)
- @files/@uploads: Upload de múltiples archivos
- @form: Form data (application/x-www-form-urlencoded)

Incluye validación de MIME type y tamaño de archivos.

Version: 0.1.0
"""

from dataclasses import dataclass, field
from typing import Any, List, Optional, Type


# ===================================
# Dataclasses
# ===================================

@dataclass
class FileMetadata:
    """
    Metadata para file uploads (multipart/form-data).
    
    Usado por decoradores @file y @files para configurar validaciones
    de archivos subidos.
    
    Attributes:
        param_name: Nombre del campo en el form-data
        mime_types: Lista de tipos MIME permitidos (vacío = todos permitidos)
        max_size: Tamaño máximo en bytes (None = sin límite)
        required: Si archivo es obligatorio
        multiple: Si acepta múltiples archivos
        description: Descripción del parámetro
    
    Example:
        # Single file con validaciones
        metadata = FileMetadata(
            param_name="document",
            mime_types=["application/pdf"],
            max_size=10_000_000,  # 10MB
            required=True
        )
        
        # Multiple files
        metadata = FileMetadata(
            param_name="images",
            mime_types=["image/jpeg", "image/png"],
            max_size=5_000_000,  # 5MB por archivo
            multiple=True
        )
    """
    
    param_name: str
    mime_types: List[str] = field(default_factory=list)
    max_size: Optional[int] = None
    required: bool = True
    multiple: bool = False
    description: Optional[str] = None
    
    def __post_init__(self):
        """Validaciones post-init."""
        if not self.param_name or not self.param_name.strip():
            raise ValueError("FileMetadata param_name cannot be empty")
        
        self.param_name = self.param_name.strip()
        
        if self.max_size is not None and self.max_size <= 0:
            raise ValueError("FileMetadata max_size must be positive")
        
        # Normalizar MIME types
        self.mime_types = [mime.strip().lower() for mime in self.mime_types]
    
    def validate_mime_type(self, mime_type: str) -> bool:
        """
        Valida que MIME type esté permitido.
        
        Args:
            mime_type: MIME type a validar (ej: "image/jpeg")
        
        Returns:
            True si está permitido o si no hay restricciones
        
        Example:
            metadata = FileMetadata("file", mime_types=["image/jpeg", "image/png"])
            metadata.validate_mime_type("image/jpeg")  # True
            metadata.validate_mime_type("application/pdf")  # False
        """
        if not self.mime_types:
            return True
        
        mime_normalized = mime_type.strip().lower()
        
        for allowed_mime in self.mime_types:
            # Soporte para wildcards (ej: "image/*")
            if '*' in allowed_mime:
                prefix = allowed_mime.split('/')[0]
                if mime_normalized.startswith(prefix + '/'):
                    return True
            elif mime_normalized == allowed_mime:
                return True
        
        return False
    
    def validate_size(self, size: int) -> bool:
        """
        Valida que tamaño esté dentro del límite.
        
        Args:
            size: Tamaño en bytes
        
        Returns:
            True si está dentro del límite o si no hay límite
        
        Example:
            metadata = FileMetadata("file", max_size=1_000_000)  # 1MB
            metadata.validate_size(500_000)  # True
            metadata.validate_size(2_000_000)  # False
        """
        if self.max_size is None:
            return True
        
        return size <= self.max_size
    
    def get_max_size_mb(self) -> Optional[float]:
        """Obtiene tamaño máximo en MB (para display)."""
        if self.max_size is None:
            return None
        return self.max_size / (1024 * 1024)


@dataclass
class FormMetadata:
    """
    Metadata para form data (application/x-www-form-urlencoded).
    
    Diferencia entre form data tradicional y JSON body.
    Usado con formularios HTML tradicionales.
    
    Attributes:
        param_name: Nombre del campo en el form (puede ser vacío si se inferirá del parámetro)
        param_type: Tipo esperado del parámetro
        required: Si parámetro es obligatorio
        default: Valor por defecto si no se provee
        description: Descripción del parámetro
    
    Example:
        metadata = FormMetadata(
            param_name="username",
            param_type=str,
            required=True,
            description="Username for login"
        )
    """
    
    param_name: str = ""
    param_type: Type = str
    required: bool = True
    default: Any = None
    description: Optional[str] = None
    
    def __post_init__(self):
        """Validaciones post-init."""
        # param_name puede ser vacío (se inferirá del nombre del parámetro en runtime)
        if self.param_name:
            self.param_name = self.param_name.strip()
        
        # Auto-set required=False cuando hay default
        if self.default is not None and self.required:
            self.required = False


# ===================================
# Marker Classes
# ===================================

class FileMarker:
    """
    Marker class para transportar FileMetadata.
    
    Se usa como default value en parámetros decorados con @file o @files.
    """
    
    def __init__(self, metadata: FileMetadata):
        self.__file_metadata__ = metadata
    
    def __repr__(self) -> str:
        return f"FileMarker(param_name={self.__file_metadata__.param_name})"


class FormMarker:
    """
    Marker class para transportar FormMetadata.
    
    Se usa como default value en parámetros decorados con @form.
    """
    
    def __init__(self, metadata: FormMetadata):
        self.__form_metadata__ = metadata
    
    def __repr__(self) -> str:
        return f"FormMarker(param_name={self.__form_metadata__.param_name})"


# ===================================
# Decorators
# ===================================

def file(
    name: str,
    *,
    mime_types: Optional[List[str]] = None,
    max_size: Optional[int] = None,
    required: bool = True,
    description: Optional[str] = None
):
    """
    Marca parámetro como FILE UPLOAD (single file).
    
    Maneja uploads multipart/form-data con validación de MIME type y tamaño.
    
    Args:
        name: Nombre del campo en el form-data
        mime_types: Tipos MIME permitidos (ej: ["image/jpeg", "image/png"])
                   Soporta wildcards (ej: ["image/*"])
                   Vacío = todos permitidos
        max_size: Tamaño máximo en bytes (ej: 5_000_000 = 5MB)
                 None = sin límite
        required: Si archivo es obligatorio
        description: Descripción del parámetro
    
    Returns:
        FileMarker con metadata de validación
    
    Examples:
        # Upload de PDF con límite de 10MB
        @post("/upload-document")
        def upload_document(
            @file("document", mime_types=["application/pdf"], max_size=10_000_000) file: File
        ):
            return save_file(file)
        
        # Upload de imagen con wildcards
        @post("/upload-image")
        def upload_image(
            @file("image", mime_types=["image/*"], max_size=5_000_000) image: File
        ):
            return save_image(image)
        
        # Upload sin restricciones
        @post("/upload-any")
        def upload_any(@file("file") file: File):
            return save_file(file)
    
    Notes:
        - En runtime, el router HTTP extraerá el archivo del request
        - Validaciones se ejecutan antes de llamar al handler
        - Si validación falla, retorna 400 Bad Request
    """
    metadata = FileMetadata(
        param_name=name,
        mime_types=mime_types or [],
        max_size=max_size,
        required=required,
        multiple=False,
        description=description
    )
    return FileMarker(metadata)


def files(
    name: str,
    *,
    mime_types: Optional[List[str]] = None,
    max_size: Optional[int] = None,
    required: bool = True,
    description: Optional[str] = None
):
    """
    Marca parámetro como MULTIPLE FILE UPLOADS.
    
    Similar a @file pero acepta múltiples archivos en un array.
    
    Args:
        name: Nombre del campo en el form-data
        mime_types: Tipos MIME permitidos para CADA archivo
        max_size: Tamaño máximo en bytes para CADA archivo
        required: Si al menos un archivo es obligatorio
        description: Descripción del parámetro
    
    Returns:
        FileMarker con metadata de validación (multiple=True)
    
    Examples:
        # Upload de múltiples imágenes
        @post("/upload-images")
        def upload_images(
            @files("images", mime_types=["image/*"], max_size=5_000_000) images: List[File]
        ):
            return save_images(images)
        
        # Upload de documentos
        @post("/upload-documents")
        def upload_documents(
            @files("documents", mime_types=["application/pdf", "application/msword"]) docs: List[File]
        ):
            return save_documents(docs)
    
    Notes:
        - Validaciones se aplican a CADA archivo individualmente
        - Si algún archivo falla validación, retorna 400 Bad Request
        - Router HTTP retorna List[File] al handler
    """
    metadata = FileMetadata(
        param_name=name,
        mime_types=mime_types or [],
        max_size=max_size,
        required=required,
        multiple=True,
        description=description
    )
    return FileMarker(metadata)


def form(
    name: Optional[str] = None,
    param_type: Type = str,
    required: bool = True,
    default: Any = None,
    description: Optional[str] = None
):
    """
    Marca parámetro como FORM DATA (application/x-www-form-urlencoded).
    
    Diferencia entre form data tradicional (HTML forms) y JSON body.
    
    Args:
        name: Nombre del campo en el form (None = usar nombre del parámetro)
        param_type: Tipo esperado del parámetro (str, int, float, bool)
        required: Si parámetro es obligatorio
        default: Valor por defecto si no se provee
        description: Descripción del parámetro
    
    Returns:
        FormMarker con metadata de validación
    
    Examples:
        # Login form tradicional
        @post("/login")
        def login(
            @form username: str,
            @form password: str
        ):
            return authenticate(username, password)
        
        # Form con defaults
        @post("/register")
        def register(
            @form username: str,
            @form email: str,
            @form("age", int, default=18) age: int,
            @form("newsletter", bool, default=False) newsletter: bool
        ):
            return create_user(username, email, age, newsletter)
        
        # Form con custom names
        @post("/search")
        def search(
            @form("q", description="Search query") query: str,
            @form("p", int, default=1) page: int
        ):
            return search_results(query, page)
    
    Notes:
        - Content-Type debe ser application/x-www-form-urlencoded
        - Diferente de @body que espera JSON (application/json)
        - Router HTTP parsea form data automáticamente
    """
    metadata = FormMetadata(
        param_name=name or "",
        param_type=param_type,
        required=required,
        default=default,
        description=description
    )
    return FormMarker(metadata)


# ===================================
# Aliases
# ===================================

upload = file
uploads = files


# ===================================
# Helper Functions
# ===================================

def is_file_parameter(marker: Any) -> bool:
    """Verifica si marker es FileMarker."""
    return isinstance(marker, FileMarker)


def is_form_parameter(marker: Any) -> bool:
    """Verifica si marker es FormMarker."""
    return isinstance(marker, FormMarker)


def get_file_metadata(marker: FileMarker) -> Optional[FileMetadata]:
    """Obtiene FileMetadata de un FileMarker."""
    if not is_file_parameter(marker):
        return None
    return marker.__file_metadata__


def get_form_metadata(marker: FormMarker) -> Optional[FormMetadata]:
    """Obtiene FormMetadata de un FormMarker."""
    if not is_form_parameter(marker):
        return None
    return marker.__form_metadata__


# ===================================
# Inline Tests (if __name__ == "__main__")
# ===================================

if __name__ == "__main__":
    # Test FileMetadata
    file_meta = FileMetadata(
        param_name="document",
        mime_types=["application/pdf"],
        max_size=10_000_000
    )
    assert file_meta.param_name == "document"
    assert file_meta.validate_mime_type("application/pdf") is True
    assert file_meta.validate_mime_type("image/jpeg") is False
    assert file_meta.validate_size(5_000_000) is True
    assert file_meta.validate_size(15_000_000) is False
    print("✓ FileMetadata tests passed")
    
    # Test wildcard MIME types
    image_meta = FileMetadata(
        param_name="image",
        mime_types=["image/*"]
    )
    assert image_meta.validate_mime_type("image/jpeg") is True
    assert image_meta.validate_mime_type("image/png") is True
    assert image_meta.validate_mime_type("application/pdf") is False
    print("✓ Wildcard MIME types tests passed")
    
    # Test FormMetadata
    form_meta = FormMetadata(
        param_name="username",
        param_type=str,
        required=True
    )
    assert form_meta.param_name == "username"
    assert form_meta.required is True
    print("✓ FormMetadata tests passed")
    
    # Test @file decorator
    file_marker = file("document", mime_types=["application/pdf"])
    assert isinstance(file_marker, FileMarker)
    assert file_marker.__file_metadata__.param_name == "document"
    print("✓ @file decorator tests passed")
    
    # Test @files decorator
    files_marker = files("images", mime_types=["image/*"])
    assert isinstance(files_marker, FileMarker)
    assert files_marker.__file_metadata__.multiple is True
    print("✓ @files decorator tests passed")
    
    # Test @form decorator
    form_marker = form("username")
    assert isinstance(form_marker, FormMarker)
    assert form_marker.__form_metadata__.param_name == "username"
    print("✓ @form decorator tests passed")
    
    print("\n✅ All inline tests passed!")
