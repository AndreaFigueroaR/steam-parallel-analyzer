use plotters::style::RGBColor;
use std::collections::HashMap;

/// Superficie (fondo) de los gráficos.
pub const SURFACE: RGBColor = RGBColor(0xfc, 0xfc, 0xfb);
/// Tinta principal (títulos, valores).
pub const TEXT_PRIMARY: RGBColor = RGBColor(0x0b, 0x0b, 0x0b);
/// Tinta secundaria (ejes, subtítulos, leyendas).
pub const TEXT_SECONDARY: RGBColor = RGBColor(0x52, 0x51, 0x4e);
/// Líneas de grilla, recesivas.
pub const GRID: RGBColor = RGBColor(0xe4, 0xe3, 0xdf);
/// Color neutro para el agregado "otros".
pub const OTHERS: RGBColor = RGBColor(0xb4, 0xb3, 0xad);

/// Paleta categórica validada para daltonismo, en orden fijo (nunca se cicla).
pub const CATEGORICAL: [RGBColor; 8] = [
    RGBColor(0x2a, 0x78, 0xd6), // azul
    RGBColor(0xeb, 0x68, 0x34), // naranja
    RGBColor(0x1b, 0xaf, 0x7a), // aqua
    RGBColor(0xed, 0xa1, 0x00), // amarillo
    RGBColor(0xe8, 0x7b, 0xa4), // magenta
    RGBColor(0x00, 0x83, 0x00), // verde
    RGBColor(0x62, 0x50, 0xd6), // violeta
    RGBColor(0xe3, 0x49, 0x48), // rojo
];

/// Asigna a cada idioma un color estable: el color sigue a la entidad (el idioma) en todos los
/// gráficos, no a su posición en un ranking. Si hubiera más idiomas que colores, el excedente
/// se pinta con el neutro de "otros".
pub struct LanguagePalette {
    colors: HashMap<String, RGBColor>,
    order: Vec<String>,
}

impl LanguagePalette {
    pub fn new<'a>(languages: impl IntoIterator<Item = &'a str>) -> Self {
        let mut palette = LanguagePalette {
            colors: HashMap::new(),
            order: Vec::new(),
        };
        for language in languages {
            if !palette.colors.contains_key(language) {
                let color = CATEGORICAL
                    .get(palette.order.len())
                    .copied()
                    .unwrap_or(OTHERS);
                palette.colors.insert(language.to_owned(), color);
                palette.order.push(language.to_owned());
            }
        }
        palette
    }

    pub fn color_of(&self, language: &str) -> RGBColor {
        self.colors.get(language).copied().unwrap_or(OTHERS)
    }

    /// Idiomas en el orden en que se les asignó color (para las leyendas).
    pub fn languages(&self) -> &[String] {
        &self.order
    }
}
