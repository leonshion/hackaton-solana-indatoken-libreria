use anchor_lang::prelude::*;

#[error_code]
pub enum IndaError {
    #[msg("No autorizado para realizar esta acción.")]
    NoAutorizado,

    #[msg("Cantidad inválida — debe ser mayor a 0.")]
    CantidadInvalida,

    #[msg("El artículo no pertenece a un Autor Top.")]
    ArticuloNoEsTop,

    #[msg("El autor aún no alcanzó los 15 artículos requeridos.")]
    AutorNoEstablecido,

    #[msg("Nombre demasiado largo — máximo 80 caracteres.")]
    NombreDemasiado,

    #[msg("Título demasiado largo — máximo 120 caracteres.")]
    TituloDemasiado,
}
