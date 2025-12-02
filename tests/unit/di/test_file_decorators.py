"""
Tests unitarios para file_decorators.py

Jira: TASK-035E
Historia: VELA-575

Tests para:
- FileMetadata: Validaciones de MIME type y size
- FormMetadata: Metadata de form data
- @file decorator: Single file upload
- @files decorator: Multiple files upload
- @form decorator: Form data parameters
- Helper functions: is_file_parameter, get_file_metadata, etc.
"""

import pytest
from src.runtime.di.file_decorators import (
    FileMetadata,
    FormMetadata,
    FileMarker,
    FormMarker,
    file,
    files,
    upload,
    uploads,
    form,
    is_file_parameter,
    is_form_parameter,
    get_file_metadata,
    get_form_metadata,
)


# ===================================
# TestFileMetadata
# ===================================

class TestFileMetadata:
    """Tests para FileMetadata dataclass."""
    
    def test_metadata_creation(self):
        """Test creación básica de FileMetadata."""
        metadata = FileMetadata(param_name="file")
        assert metadata.param_name == "file"
        assert metadata.mime_types == []
        assert metadata.max_size is None
        assert metadata.required is True
        assert metadata.multiple is False
    
    def test_metadata_with_all_fields(self):
        """Test creación con todos los campos."""
        metadata = FileMetadata(
            param_name="document",
            mime_types=["application/pdf"],
            max_size=10_000_000,
            required=True,
            multiple=False,
            description="PDF document"
        )
        assert metadata.param_name == "document"
        assert metadata.mime_types == ["application/pdf"]
        assert metadata.max_size == 10_000_000
        assert metadata.description == "PDF document"
    
    def test_metadata_validates_empty_param_name(self):
        """Test validación de param_name vacío."""
        with pytest.raises(ValueError, match="param_name cannot be empty"):
            FileMetadata(param_name="")
        
        with pytest.raises(ValueError, match="param_name cannot be empty"):
            FileMetadata(param_name="   ")
    
    def test_metadata_validates_max_size_positive(self):
        """Test validación de max_size positivo."""
        with pytest.raises(ValueError, match="max_size must be positive"):
            FileMetadata(param_name="file", max_size=0)
        
        with pytest.raises(ValueError, match="max_size must be positive"):
            FileMetadata(param_name="file", max_size=-100)
    
    def test_metadata_normalizes_mime_types(self):
        """Test normalización de MIME types (lowercase, trim)."""
        metadata = FileMetadata(
            param_name="file",
            mime_types=["  IMAGE/JPEG  ", "Application/PDF"]
        )
        assert metadata.mime_types == ["image/jpeg", "application/pdf"]
    
    def test_validate_mime_type_exact_match(self):
        """Test validación de MIME type exacto."""
        metadata = FileMetadata(
            param_name="file",
            mime_types=["application/pdf", "image/jpeg"]
        )
        assert metadata.validate_mime_type("application/pdf") is True
        assert metadata.validate_mime_type("image/jpeg") is True
        assert metadata.validate_mime_type("image/png") is False
    
    def test_validate_mime_type_wildcard(self):
        """Test validación de MIME type con wildcard."""
        metadata = FileMetadata(
            param_name="file",
            mime_types=["image/*"]
        )
        assert metadata.validate_mime_type("image/jpeg") is True
        assert metadata.validate_mime_type("image/png") is True
        assert metadata.validate_mime_type("image/gif") is True
        assert metadata.validate_mime_type("application/pdf") is False
    
    def test_validate_mime_type_no_restrictions(self):
        """Test validación sin restricciones (vacío = todos permitidos)."""
        metadata = FileMetadata(param_name="file", mime_types=[])
        assert metadata.validate_mime_type("application/pdf") is True
        assert metadata.validate_mime_type("image/jpeg") is True
        assert metadata.validate_mime_type("text/plain") is True
    
    def test_validate_size_within_limit(self):
        """Test validación de tamaño dentro del límite."""
        metadata = FileMetadata(param_name="file", max_size=5_000_000)
        assert metadata.validate_size(1_000_000) is True
        assert metadata.validate_size(5_000_000) is True  # exacto OK
        assert metadata.validate_size(10_000_000) is False
    
    def test_validate_size_no_limit(self):
        """Test validación sin límite de tamaño."""
        metadata = FileMetadata(param_name="file", max_size=None)
        assert metadata.validate_size(1_000_000) is True
        assert metadata.validate_size(1_000_000_000) is True  # 1GB
    
    def test_get_max_size_mb(self):
        """Test conversión de max_size a MB."""
        metadata = FileMetadata(param_name="file", max_size=5_000_000)
        assert metadata.get_max_size_mb() == pytest.approx(4.76837158)  # ~5MB
        
        metadata_no_limit = FileMetadata(param_name="file", max_size=None)
        assert metadata_no_limit.get_max_size_mb() is None


# ===================================
# TestFormMetadata
# ===================================

class TestFormMetadata:
    """Tests para FormMetadata dataclass."""
    
    def test_metadata_creation(self):
        """Test creación básica de FormMetadata."""
        metadata = FormMetadata(param_name="username")
        assert metadata.param_name == "username"
        assert metadata.param_type == str
        assert metadata.required is True
        assert metadata.default is None
    
    def test_metadata_with_all_fields(self):
        """Test creación con todos los campos."""
        metadata = FormMetadata(
            param_name="age",
            param_type=int,
            required=False,
            default=18,
            description="User age"
        )
        assert metadata.param_name == "age"
        assert metadata.param_type == int
        assert metadata.required is False
        assert metadata.default == 18
    
    def test_metadata_allows_empty_param_name(self):
        """Test que param_name vacío es permitido (se inferirá del parámetro)."""
        metadata = FormMetadata(param_name="")
        assert metadata.param_name == ""
        assert metadata.param_type == str
        assert metadata.required is True
    
    def test_metadata_auto_sets_required_false_with_default(self):
        """Test auto-set required=False cuando hay default."""
        metadata = FormMetadata(
            param_name="newsletter",
            default=False,
            required=True  # se auto-setea a False
        )
        assert metadata.required is False


# ===================================
# TestFileDecorator
# ===================================

class TestFileDecorator:
    """Tests para @file decorator."""
    
    def test_file_basic(self):
        """Test @file básico sin opciones."""
        marker = file("document")
        assert isinstance(marker, FileMarker)
        assert marker.__file_metadata__.param_name == "document"
        assert marker.__file_metadata__.mime_types == []
        assert marker.__file_metadata__.max_size is None
        assert marker.__file_metadata__.required is True
        assert marker.__file_metadata__.multiple is False
    
    def test_file_with_mime_types(self):
        """Test @file con MIME types."""
        marker = file("document", mime_types=["application/pdf"])
        assert marker.__file_metadata__.mime_types == ["application/pdf"]
    
    def test_file_with_max_size(self):
        """Test @file con max_size."""
        marker = file("document", max_size=10_000_000)
        assert marker.__file_metadata__.max_size == 10_000_000
    
    def test_file_with_all_options(self):
        """Test @file con todas las opciones."""
        marker = file(
            "document",
            mime_types=["application/pdf", "application/msword"],
            max_size=10_000_000,
            required=True,
            description="Upload document"
        )
        assert marker.__file_metadata__.param_name == "document"
        assert marker.__file_metadata__.mime_types == ["application/pdf", "application/msword"]
        assert marker.__file_metadata__.max_size == 10_000_000
        assert marker.__file_metadata__.required is True
        assert marker.__file_metadata__.description == "Upload document"
    
    def test_upload_alias(self):
        """Test que upload() es alias de file()."""
        marker = upload("document")
        assert isinstance(marker, FileMarker)
        assert marker.__file_metadata__.param_name == "document"


# ===================================
# TestFilesDecorator
# ===================================

class TestFilesDecorator:
    """Tests para @files decorator (múltiples archivos)."""
    
    def test_files_basic(self):
        """Test @files básico."""
        marker = files("images")
        assert isinstance(marker, FileMarker)
        assert marker.__file_metadata__.param_name == "images"
        assert marker.__file_metadata__.multiple is True
    
    def test_files_with_mime_types(self):
        """Test @files con MIME types."""
        marker = files("images", mime_types=["image/*"])
        assert marker.__file_metadata__.mime_types == ["image/*"]
        assert marker.__file_metadata__.multiple is True
    
    def test_files_with_max_size(self):
        """Test @files con max_size (por archivo)."""
        marker = files("images", max_size=5_000_000)
        assert marker.__file_metadata__.max_size == 5_000_000
        assert marker.__file_metadata__.multiple is True
    
    def test_files_with_all_options(self):
        """Test @files con todas las opciones."""
        marker = files(
            "documents",
            mime_types=["application/pdf"],
            max_size=10_000_000,
            required=True,
            description="Upload multiple documents"
        )
        assert marker.__file_metadata__.param_name == "documents"
        assert marker.__file_metadata__.multiple is True
    
    def test_uploads_alias(self):
        """Test que uploads() es alias de files()."""
        marker = uploads("images")
        assert isinstance(marker, FileMarker)
        assert marker.__file_metadata__.multiple is True


# ===================================
# TestFormDecorator
# ===================================

class TestFormDecorator:
    """Tests para @form decorator."""
    
    def test_form_basic(self):
        """Test @form básico."""
        marker = form()
        assert isinstance(marker, FormMarker)
        assert marker.__form_metadata__.param_name == ""
        assert marker.__form_metadata__.param_type == str
    
    def test_form_with_name(self):
        """Test @form con nombre."""
        marker = form("username")
        assert marker.__form_metadata__.param_name == "username"
    
    def test_form_with_type(self):
        """Test @form con tipo."""
        marker = form("age", param_type=int)
        assert marker.__form_metadata__.param_type == int
    
    def test_form_with_default(self):
        """Test @form con default."""
        marker = form("newsletter", param_type=bool, default=False)
        assert marker.__form_metadata__.default is False
        assert marker.__form_metadata__.required is False  # auto-seteado
    
    def test_form_with_all_options(self):
        """Test @form con todas las opciones."""
        marker = form(
            "page",
            param_type=int,
            required=False,
            default=1,
            description="Page number"
        )
        assert marker.__form_metadata__.param_name == "page"
        assert marker.__form_metadata__.param_type == int
        assert marker.__form_metadata__.default == 1
        assert marker.__form_metadata__.description == "Page number"


# ===================================
# TestHelperFunctions
# ===================================

class TestHelperFunctions:
    """Tests para helper functions."""
    
    def test_is_file_parameter_true(self):
        """Test is_file_parameter retorna True para FileMarker."""
        marker = file("document")
        assert is_file_parameter(marker) is True
    
    def test_is_file_parameter_false(self):
        """Test is_file_parameter retorna False para no FileMarker."""
        assert is_file_parameter("string") is False
        assert is_file_parameter(123) is False
        assert is_file_parameter(form("username")) is False
    
    def test_is_form_parameter_true(self):
        """Test is_form_parameter retorna True para FormMarker."""
        marker = form("username")
        assert is_form_parameter(marker) is True
    
    def test_is_form_parameter_false(self):
        """Test is_form_parameter retorna False para no FormMarker."""
        assert is_form_parameter("string") is False
        assert is_form_parameter(123) is False
        assert is_form_parameter(file("document")) is False
    
    def test_get_file_metadata_exists(self):
        """Test get_file_metadata retorna metadata."""
        marker = file("document", mime_types=["application/pdf"])
        metadata = get_file_metadata(marker)
        assert metadata is not None
        assert metadata.param_name == "document"
    
    def test_get_file_metadata_not_exists(self):
        """Test get_file_metadata retorna None para no FileMarker."""
        assert get_file_metadata("string") is None
        assert get_file_metadata(123) is None
    
    def test_get_form_metadata_exists(self):
        """Test get_form_metadata retorna metadata."""
        marker = form("username")
        metadata = get_form_metadata(marker)
        assert metadata is not None
        assert metadata.param_name == "username"
    
    def test_get_form_metadata_not_exists(self):
        """Test get_form_metadata retorna None para no FormMarker."""
        assert get_form_metadata("string") is None
        assert get_form_metadata(123) is None


# ===================================
# TestIntegration
# ===================================

class TestIntegration:
    """Tests de integración end-to-end."""
    
    def test_complete_file_upload_flow(self):
        """Test flujo completo de file upload con validaciones."""
        # 1. Crear marker con @file
        marker = file(
            "document",
            mime_types=["application/pdf"],
            max_size=10_000_000
        )
        
        # 2. Verificar que es file parameter
        assert is_file_parameter(marker) is True
        
        # 3. Extraer metadata
        metadata = get_file_metadata(marker)
        assert metadata is not None
        
        # 4. Validar MIME type
        assert metadata.validate_mime_type("application/pdf") is True
        assert metadata.validate_mime_type("image/jpeg") is False
        
        # 5. Validar size
        assert metadata.validate_size(5_000_000) is True
        assert metadata.validate_size(15_000_000) is False
    
    def test_multiple_files_upload_flow(self):
        """Test flujo de múltiples archivos."""
        marker = files(
            "images",
            mime_types=["image/*"],
            max_size=5_000_000
        )
        
        metadata = get_file_metadata(marker)
        assert metadata.multiple is True
        
        # Validar varios tipos de imágenes
        assert metadata.validate_mime_type("image/jpeg") is True
        assert metadata.validate_mime_type("image/png") is True
        assert metadata.validate_mime_type("image/gif") is True
    
    def test_form_data_flow(self):
        """Test flujo de form data."""
        username_marker = form("username")
        age_marker = form("age", param_type=int, default=18)
        
        assert is_form_parameter(username_marker) is True
        assert is_form_parameter(age_marker) is True
        
        username_meta = get_form_metadata(username_marker)
        age_meta = get_form_metadata(age_marker)
        
        assert username_meta.required is True
        assert age_meta.required is False  # auto-seteado por default
    
    def test_mixed_decorators_in_endpoint(self):
        """Test uso mezclado de @file y @form en mismo endpoint."""
        # Simular endpoint con file + form data
        document_marker = file("document", mime_types=["application/pdf"])
        title_marker = form("title")
        description_marker = form("description", required=False, default="")
        
        # Verificar tipos
        assert is_file_parameter(document_marker) is True
        assert is_form_parameter(title_marker) is True
        assert is_form_parameter(description_marker) is True
        
        # Extraer metadata
        doc_meta = get_file_metadata(document_marker)
        title_meta = get_form_metadata(title_marker)
        desc_meta = get_form_metadata(description_marker)
        
        assert doc_meta.param_name == "document"
        assert title_meta.param_name == "title"
        assert desc_meta.param_name == "description"
        assert desc_meta.required is False


# ===================================
# TestEdgeCases
# ===================================

class TestEdgeCases:
    """Tests de casos edge."""
    
    def test_file_with_empty_mime_types(self):
        """Test @file con mime_types vacío (todos permitidos)."""
        marker = file("file", mime_types=[])
        metadata = get_file_metadata(marker)
        assert metadata.validate_mime_type("application/pdf") is True
        assert metadata.validate_mime_type("text/plain") is True
    
    def test_file_with_no_max_size(self):
        """Test @file sin límite de tamaño."""
        marker = file("file", max_size=None)
        metadata = get_file_metadata(marker)
        assert metadata.validate_size(1_000_000_000) is True  # 1GB
    
    def test_form_with_none_name(self):
        """Test @form con name=None (se setea vacío)."""
        marker = form(name=None)
        metadata = get_form_metadata(marker)
        assert metadata.param_name == ""
    
    def test_wildcard_mime_type_variations(self):
        """Test wildcards en diferentes formatos."""
        marker = file("file", mime_types=["image/*", "video/*"])
        metadata = get_file_metadata(marker)
        
        assert metadata.validate_mime_type("image/jpeg") is True
        assert metadata.validate_mime_type("video/mp4") is True
        assert metadata.validate_mime_type("audio/mp3") is False
    
    def test_file_marker_repr(self):
        """Test __repr__ de FileMarker."""
        marker = file("document")
        repr_str = repr(marker)
        assert "FileMarker" in repr_str
        assert "document" in repr_str
    
    def test_form_marker_repr(self):
        """Test __repr__ de FormMarker."""
        marker = form("username")
        repr_str = repr(marker)
        assert "FormMarker" in repr_str
        assert "username" in repr_str


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
