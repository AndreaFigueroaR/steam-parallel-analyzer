//! Generación de gráficos SVG a partir de las métricas obtenidas.
//!
//! Los gráficos se dibujan con `plotters` usando únicamente su backend SVG (sin dependencias
//! nativas ni fuentes del sistema), por lo que pueden versionarse y verse directamente en GitHub.

mod palette;

use crate::dtos::program_output_dto::ProgramOutputDto;
use palette::*;
use plotters::coord::Shift;
use plotters::prelude::*;
use plotters::style::text_anchor::{HPos, Pos, VPos};
use std::error::Error;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const WIDTH: u32 = 960;
const FONT: &str = "sans-serif";
const HEADER_HEIGHT: u32 = 96;
const BAR_ROW_HEIGHT: u32 = 64;
const LABEL_OTHER_LANGUAGES: &str = "otros idiomas";

type ChartResult = Result<(), Box<dyn Error>>;

/// Una medición del benchmark de escalabilidad: tiempo de procesamiento con `threads` workers.
pub struct ScalingSample {
    pub threads: usize,
    pub seconds: f64,
}

/// Barra horizontal genérica: etiqueta, valor, color y texto a mostrar junto a la barra.
struct Bar {
    label: String,
    value: f64,
    color: RGBColor,
    value_label: String,
}

/// Encabezado común: título, subtítulo y, opcionalmente, una leyenda en una fila.
struct Header<'a> {
    title: &'a str,
    subtitle: &'a str,
    legend: Vec<(String, RGBColor)>,
}

/// Responsable de dibujar los gráficos en un directorio de salida.
pub struct ChartsRenderer {
    directory: PathBuf,
}

impl ChartsRenderer {
    pub const TOP_GAMES: &str = "top_games.svg";
    pub const GAMES_LANGUAGES: &str = "games_languages.svg";
    pub const TOP_LANGUAGES: &str = "top_languages.svg";
    pub const TOP_REVIEWS_VOTES: &str = "top_languages_reviews_votes.svg";
    pub const SCALING_TIME: &str = "scaling_time.svg";
    pub const SCALING_SPEEDUP: &str = "scaling_speedup.svg";

    /// Crea el renderer, creando el directorio de salida si no existe.
    pub fn new(directory: &str) -> io::Result<Self> {
        fs::create_dir_all(directory)?;
        Ok(ChartsRenderer {
            directory: PathBuf::from(directory),
        })
    }

    /// Dibuja los cuatro gráficos de métricas y devuelve los nombres de los archivos generados.
    pub fn render_metrics(&self, output: &ProgramOutputDto) -> io::Result<Vec<String>> {
        let palette = LanguagePalette::new(
            output.top_languages().iter().map(|l| l.language()).chain(
                output
                    .top_games()
                    .iter()
                    .flat_map(|g| g.languages().iter().map(|l| l.language())),
            ),
        );
        self.render(Self::TOP_GAMES, |path| top_games(path, output))?;
        self.render(Self::GAMES_LANGUAGES, |path| {
            games_languages(path, output, &palette)
        })?;
        self.render(Self::TOP_LANGUAGES, |path| {
            top_languages(path, output, &palette)
        })?;
        self.render(Self::TOP_REVIEWS_VOTES, |path| {
            top_reviews_votes(path, output, &palette)
        })?;
        Ok(vec![
            Self::TOP_GAMES.into(),
            Self::GAMES_LANGUAGES.into(),
            Self::TOP_LANGUAGES.into(),
            Self::TOP_REVIEWS_VOTES.into(),
        ])
    }

    /// Dibuja los gráficos de escalabilidad (tiempo y speedup por cantidad de workers).
    pub fn render_scaling(&self, samples: &[ScalingSample]) -> io::Result<Vec<String>> {
        self.render(Self::SCALING_TIME, |path| scaling_time(path, samples))?;
        self.render(Self::SCALING_SPEEDUP, |path| scaling_speedup(path, samples))?;
        Ok(vec![
            Self::SCALING_TIME.into(),
            Self::SCALING_SPEEDUP.into(),
        ])
    }

    fn render(&self, file_name: &str, draw: impl FnOnce(&Path) -> ChartResult) -> io::Result<()> {
        draw(&self.directory.join(file_name))
            .map_err(|err| io::Error::other(format!("No se pudo generar {file_name}: {err}")))
    }
}

// ---------------------------------------------------------------------------------------------
// Gráficos de métricas
// ---------------------------------------------------------------------------------------------

fn top_games(path: &Path, output: &ProgramOutputDto) -> ChartResult {
    let bars = output
        .top_games()
        .iter()
        .map(|game| Bar {
            label: game.game().to_owned(),
            value: game.review_count() as f64,
            color: CATEGORICAL[0],
            value_label: format_thousands(game.review_count() as u64),
        })
        .collect::<Vec<_>>();
    horizontal_bars(
        path,
        &Header {
            title: "Juegos con más reviews",
            subtitle: "Cantidad total de reviews por juego",
            legend: Vec::new(),
        },
        "reviews",
        &bars,
    )
}

fn top_languages(path: &Path, output: &ProgramOutputDto, palette: &LanguagePalette) -> ChartResult {
    let bars = output
        .top_languages()
        .iter()
        .map(|language| Bar {
            label: language.language().to_owned(),
            value: language.review_count() as f64,
            color: palette.color_of(language.language()),
            value_label: format_thousands(language.review_count() as u64),
        })
        .collect::<Vec<_>>();
    horizontal_bars(
        path,
        &Header {
            title: "Idiomas con más reviews",
            subtitle: "Cantidad total de reviews por idioma, considerando todos los juegos",
            legend: Vec::new(),
        },
        "reviews",
        &bars,
    )
}

/// Barras apiladas: para cada top juego, cuántas reviews aporta cada uno de sus top idiomas
/// y cuántas el resto de los idiomas.
fn games_languages(
    path: &Path,
    output: &ProgramOutputDto,
    palette: &LanguagePalette,
) -> ChartResult {
    let games = output.top_games();
    let rows = games.len().max(1) as u32;
    let root = SVGBackend::new(path, (WIDTH, HEADER_HEIGHT + 60 + rows * BAR_ROW_HEIGHT))
        .into_drawing_area();
    root.fill(&SURFACE)?;

    let mut legend: Vec<(String, RGBColor)> = palette
        .languages()
        .iter()
        .filter(|language| {
            games.iter().any(|g| {
                g.languages()
                    .iter()
                    .any(|l| l.language() == language.as_str())
            })
        })
        .map(|language| (language.clone(), palette.color_of(language)))
        .collect();
    legend.push((LABEL_OTHER_LANGUAGES.to_owned(), OTHERS));
    let body = draw_header(
        &root,
        &Header {
            title: "Reviews por idioma en los juegos más reseñados",
            subtitle: "Aporte de los idiomas principales de cada juego sobre su total de reviews",
            legend,
        },
    )?;

    let x_max = games.iter().map(|g| g.review_count()).max().unwrap_or(1) as f64 * 1.15;
    let mut chart = ChartBuilder::on(&body)
        .margin_left(16)
        .margin_right(32)
        .x_label_area_size(40)
        .y_label_area_size(label_area_width(games.iter().map(|g| g.game())))
        .build_cartesian_2d(
            0f64..x_max, // Los rangos enteros de plotters incluyen el extremo: `rows - 1` deja exactamente `rows` filas.
            (0..rows - 1).into_segmented(),
        )?;
    configure_horizontal_mesh(&mut chart, "reviews", |row| {
        games
            .get((rows - 1 - row) as usize)
            .map(|g| g.game().to_owned())
            .unwrap_or_default()
    })?;

    for (rank, game) in games.iter().enumerate() {
        let row = rows - 1 - rank as u32;
        let mut start = 0f64;
        let mut segments: Vec<(f64, RGBColor)> = game
            .languages()
            .iter()
            .map(|l| (l.review_count() as f64, palette.color_of(l.language())))
            .collect();
        let others = game.review_count() as f64 - segments.iter().map(|(v, _)| v).sum::<f64>();
        if others > 0.0 {
            segments.push((others, OTHERS));
        }
        for (value, color) in segments {
            let end = start + value;
            let corners = [
                (start, SegmentValue::Exact(row)),
                (end, SegmentValue::Exact(row + 1)),
            ];
            chart.draw_series([
                bar_rectangle(corners.clone(), color.filled()),
                // Separador de 2px del color de la superficie entre segmentos.
                bar_rectangle(corners, SURFACE.stroke_width(2)),
            ])?;
            start = end;
        }
        draw_value_label(
            &mut chart,
            (start, SegmentValue::CenterOf(row)),
            format_thousands(game.review_count() as u64),
        )?;
    }
    root.present()?;
    Ok(())
}

/// Votos de las top reviews de cada idioma principal, por posición en el ranking.
fn top_reviews_votes(
    path: &Path,
    output: &ProgramOutputDto,
    palette: &LanguagePalette,
) -> ChartResult {
    let languages = output.top_languages();
    let root = SVGBackend::new(path, (WIDTH, 560)).into_drawing_area();
    root.fill(&SURFACE)?;
    let legend = languages
        .iter()
        .map(|l| (l.language().to_owned(), palette.color_of(l.language())))
        .collect();
    let body = draw_header(
        &root,
        &Header {
            title: "Votos de utilidad de las reviews más votadas",
            subtitle: "Votos \"útil\" de las top reviews de cada idioma principal, según su posición",
            legend,
        },
    )?;

    let max_rank = languages
        .iter()
        .map(|l| l.top_reviews().len())
        .max()
        .unwrap_or(1)
        .max(1);
    let y_max = languages
        .iter()
        .flat_map(|l| l.top_reviews().iter().map(|r| r.votes))
        .max()
        .unwrap_or(1)
        .max(1) as f64
        * 1.1;
    let mut chart = ChartBuilder::on(&body)
        .margin_left(16)
        .margin_right(96)
        .margin_top(8)
        .x_label_area_size(44)
        .y_label_area_size(72)
        .build_cartesian_2d(0.5f64..max_rank as f64 + 0.5, 0f64..y_max)?;
    chart
        .configure_mesh()
        .disable_x_mesh()
        .light_line_style(TRANSPARENT)
        .bold_line_style(GRID.stroke_width(1))
        .axis_style(GRID.stroke_width(1))
        .x_labels(max_rank)
        .x_label_formatter(&|x| format!("#{}", x.round() as usize))
        .y_label_formatter(&|y| format_thousands(y.round() as u64))
        .x_desc("posición de la review en el ranking del idioma")
        .y_desc("votos útil")
        .label_style((FONT, 13).into_font().color(&TEXT_SECONDARY))
        .axis_desc_style((FONT, 13).into_font().color(&TEXT_SECONDARY))
        .draw()?;

    for language in languages {
        let color = palette.color_of(language.language());
        let points: Vec<(f64, f64)> = language
            .top_reviews()
            .iter()
            .enumerate()
            .map(|(i, review)| ((i + 1) as f64, review.votes as f64))
            .collect();
        chart.draw_series(LineSeries::new(points.clone(), color.stroke_width(2)))?;
        chart.draw_series(
            points
                .iter()
                .map(|&p| Circle::new(p, 4, ShapeStyle::from(&color).filled())),
        )?;
        if let Some(&(x, y)) = points.last() {
            chart.draw_series(std::iter::once(
                EmptyElement::at((x, y))
                    + Text::new(
                        language.language().to_owned(),
                        (10, 0),
                        (FONT, 13)
                            .into_font()
                            .color(&TEXT_SECONDARY)
                            .pos(Pos::new(HPos::Left, VPos::Center)),
                    ),
            ))?;
        }
    }
    root.present()?;
    Ok(())
}

// ---------------------------------------------------------------------------------------------
// Gráficos de escalabilidad
// ---------------------------------------------------------------------------------------------

fn scaling_time(path: &Path, samples: &[ScalingSample]) -> ChartResult {
    let bars = samples
        .iter()
        .map(|s| Bar {
            label: format!(
                "{} {}",
                s.threads,
                if s.threads == 1 { "hilo" } else { "hilos" }
            ),
            value: s.seconds,
            color: CATEGORICAL[0],
            value_label: format!("{:.2} s", s.seconds).replace('.', ","),
        })
        .collect::<Vec<_>>();
    horizontal_bars(
        path,
        &Header {
            title: "Tiempo de procesamiento por cantidad de workers",
            subtitle: "Mejor tiempo de varias corridas (lectura + procesamiento fork-join)",
            legend: Vec::new(),
        },
        "segundos",
        &bars,
    )
}

fn scaling_speedup(path: &Path, samples: &[ScalingSample]) -> ChartResult {
    let base = samples
        .iter()
        .find(|s| s.threads == 1)
        .or(samples.first())
        .map(|s| s.seconds)
        .ok_or("no hay mediciones")?;
    let max_threads = samples.iter().map(|s| s.threads).max().unwrap_or(1) as f64;
    let points: Vec<(f64, f64)> = samples
        .iter()
        .map(|s| (s.threads as f64, base / s.seconds))
        .collect();
    let y_max = points
        .iter()
        .map(|p| p.1)
        .fold(max_threads, f64::max)
        .max(1.0)
        * 1.1;

    let root = SVGBackend::new(path, (WIDTH, 560)).into_drawing_area();
    root.fill(&SURFACE)?;
    let body = draw_header(
        &root,
        &Header {
            title: "Speedup respecto de la ejecución con 1 hilo",
            subtitle: "Tiempo con 1 hilo dividido el tiempo con N hilos",
            legend: vec![
                ("speedup medido".to_owned(), CATEGORICAL[0]),
                ("speedup ideal (lineal)".to_owned(), OTHERS),
            ],
        },
    )?;
    let mut chart = ChartBuilder::on(&body)
        .margin_left(16)
        .margin_right(48)
        .margin_top(8)
        .x_label_area_size(44)
        .y_label_area_size(56)
        .build_cartesian_2d(0f64..max_threads + 1.0, 0f64..y_max)?;
    chart
        .configure_mesh()
        .light_line_style(TRANSPARENT)
        .bold_line_style(GRID.stroke_width(1))
        .axis_style(GRID.stroke_width(1))
        .x_label_formatter(&|x| format!("{}", x.round() as usize))
        .y_label_formatter(&|y| format!("{y:.0}x"))
        .x_desc("cantidad de workers")
        .y_desc("speedup")
        .label_style((FONT, 13).into_font().color(&TEXT_SECONDARY))
        .axis_desc_style((FONT, 13).into_font().color(&TEXT_SECONDARY))
        .draw()?;

    chart.draw_series(DashedLineSeries::new(
        vec![(1.0, 1.0), (max_threads, max_threads)],
        6,
        6,
        OTHERS.stroke_width(2),
    ))?;
    chart.draw_series(LineSeries::new(
        points.clone(),
        CATEGORICAL[0].stroke_width(2),
    ))?;
    chart.draw_series(points.iter().map(|&p| {
        EmptyElement::at(p)
            + Circle::new((0, 0), 4, CATEGORICAL[0].filled())
            + Text::new(
                format!("{:.2}x", p.1).replace('.', ","),
                (0, -12),
                (FONT, 12)
                    .into_font()
                    .color(&TEXT_PRIMARY)
                    .pos(Pos::new(HPos::Center, VPos::Bottom)),
            )
    }))?;
    root.present()?;
    Ok(())
}

// ---------------------------------------------------------------------------------------------
// Utilidades de dibujo
// ---------------------------------------------------------------------------------------------

/// Gráfico de barras horizontales ordenado de arriba (primer elemento) hacia abajo.
fn horizontal_bars(path: &Path, header: &Header, x_desc: &str, bars: &[Bar]) -> ChartResult {
    let rows = bars.len().max(1) as u32;
    let root = SVGBackend::new(path, (WIDTH, HEADER_HEIGHT + 60 + rows * BAR_ROW_HEIGHT))
        .into_drawing_area();
    root.fill(&SURFACE)?;
    let body = draw_header(&root, header)?;

    let x_max = bars
        .iter()
        .map(|b| b.value)
        .fold(0f64, f64::max)
        .max(f64::EPSILON)
        * 1.15;
    let mut chart = ChartBuilder::on(&body)
        .margin_left(16)
        .margin_right(32)
        .x_label_area_size(40)
        .y_label_area_size(label_area_width(bars.iter().map(|b| b.label.as_str())))
        .build_cartesian_2d(
            0f64..x_max, // Los rangos enteros de plotters incluyen el extremo: `rows - 1` deja exactamente `rows` filas.
            (0..rows - 1).into_segmented(),
        )?;
    configure_horizontal_mesh(&mut chart, x_desc, |row| {
        bars.get((rows - 1 - row) as usize)
            .map(|b| b.label.clone())
            .unwrap_or_default()
    })?;

    for (rank, bar) in bars.iter().enumerate() {
        let row = rows - 1 - rank as u32;
        chart.draw_series(std::iter::once(bar_rectangle(
            [
                (0.0, SegmentValue::Exact(row)),
                (bar.value, SegmentValue::Exact(row + 1)),
            ],
            bar.color.filled(),
        )))?;
        draw_value_label(
            &mut chart,
            (bar.value, SegmentValue::CenterOf(row)),
            bar.value_label.clone(),
        )?;
    }
    root.present()?;
    Ok(())
}

type HorizontalChart<'a, 'b> = ChartContext<
    'a,
    SVGBackend<'b>,
    Cartesian2d<
        plotters::coord::types::RangedCoordf64,
        plotters::coord::ranged1d::SegmentedCoord<plotters::coord::types::RangedCoordu32>,
    >,
>;

fn configure_horizontal_mesh(
    chart: &mut HorizontalChart,
    x_desc: &str,
    row_label: impl Fn(u32) -> String,
) -> ChartResult {
    chart
        .configure_mesh()
        .disable_y_mesh()
        .light_line_style(TRANSPARENT)
        .bold_line_style(GRID.stroke_width(1))
        .axis_style(GRID.stroke_width(1))
        .x_labels(6)
        .x_label_formatter(&|x| format_compact(*x))
        .y_label_formatter(&|segment| match segment {
            SegmentValue::CenterOf(row) => row_label(*row),
            _ => String::new(),
        })
        .x_desc(x_desc)
        .label_style((FONT, 13).into_font().color(&TEXT_SECONDARY))
        .axis_desc_style((FONT, 13).into_font().color(&TEXT_SECONDARY))
        .draw()?;
    Ok(())
}

/// Rectángulo de una barra horizontal, con aire arriba y abajo dentro de su fila.
fn bar_rectangle(
    corners: [(f64, SegmentValue<u32>); 2],
    style: ShapeStyle,
) -> Rectangle<(f64, SegmentValue<u32>)> {
    let mut rectangle = Rectangle::new(corners, style);
    rectangle.set_margin(12, 12, 0, 0);
    rectangle
}

/// Dibuja una etiqueta de valor a la derecha del extremo de una barra horizontal.
fn draw_value_label(
    chart: &mut HorizontalChart,
    coord: (f64, SegmentValue<u32>),
    label: String,
) -> ChartResult {
    chart.draw_series(std::iter::once(
        EmptyElement::at(coord)
            + Text::new(
                label,
                (8, 0),
                (FONT, 13)
                    .into_font()
                    .color(&TEXT_PRIMARY)
                    .pos(Pos::new(HPos::Left, VPos::Center)),
            ),
    ))?;
    Ok(())
}

/// Dibuja título, subtítulo y leyenda; devuelve el área restante para el gráfico.
fn draw_header<'a>(
    root: &DrawingArea<SVGBackend<'a>, Shift>,
    header: &Header,
) -> Result<DrawingArea<SVGBackend<'a>, Shift>, Box<dyn Error>> {
    let (top, body) = root.split_vertically(HEADER_HEIGHT);
    top.draw(&Text::new(
        header.title.to_owned(),
        (24, 18),
        (FONT, 21)
            .into_font()
            .style(FontStyle::Bold)
            .color(&TEXT_PRIMARY),
    ))?;
    top.draw(&Text::new(
        header.subtitle.to_owned(),
        (24, 46),
        (FONT, 14).into_font().color(&TEXT_SECONDARY),
    ))?;
    let mut x = 24;
    for (label, color) in &header.legend {
        top.draw(&Rectangle::new([(x, 72), (x + 12, 84)], color.filled()))?;
        top.draw(&Text::new(
            label.clone(),
            (x + 18, 78),
            (FONT, 13)
                .into_font()
                .color(&TEXT_SECONDARY)
                .pos(Pos::new(HPos::Left, VPos::Center)),
        ))?;
        x += 18 + estimate_text_width(label, 13) + 24;
    }
    Ok(body)
}

/// Ancho del área de etiquetas del eje Y en función de la etiqueta más larga.
fn label_area_width<'a>(labels: impl Iterator<Item = &'a str>) -> u32 {
    let longest = labels
        .map(|l| estimate_text_width(l, 13))
        .max()
        .unwrap_or(0);
    (longest + 24).clamp(80, 340) as u32
}

/// Estimación del ancho de un texto en píxeles para una fuente sans-serif.
fn estimate_text_width(text: &str, font_size: i32) -> i32 {
    (text.chars().count() as f64 * font_size as f64 * 0.6).ceil() as i32
}

/// Formatea un entero con separador de miles: 1644255 -> "1.644.255".
pub fn format_thousands(value: u64) -> String {
    let digits = value.to_string();
    let mut formatted = String::with_capacity(digits.len() + digits.len() / 3);
    for (i, digit) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            formatted.push('.');
        }
        formatted.push(digit);
    }
    formatted
}

/// Formato compacto para ticks de ejes: 1500000 -> "1,5 M", 2500 -> "2,5 k", 0.75 -> "0,75".
fn format_compact(value: f64) -> String {
    let (scaled, suffix) = if value >= 1e6 {
        (value / 1e6, " M")
    } else if value >= 1e3 {
        (value / 1e3, " k")
    } else {
        (value, "")
    };
    let text = if scaled.fract().abs() < 1e-9 || scaled >= 10.0 {
        format!("{scaled:.0}")
    } else {
        format!("{scaled:.1}")
    };
    format!("{}{suffix}", text.replace('.', ","))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thousands_separator() {
        assert_eq!(format_thousands(0), "0");
        assert_eq!(format_thousands(999), "999");
        assert_eq!(format_thousands(1_644_255), "1.644.255");
    }

    #[test]
    fn compact_ticks() {
        assert_eq!(format_compact(1_500_000.0), "1,5 M");
        assert_eq!(format_compact(2_000.0), "2 k");
        assert_eq!(format_compact(0.5), "0,5");
    }
}
