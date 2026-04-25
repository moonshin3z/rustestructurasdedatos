// motor de busqueda simple en rust
// estructuras usadas:
//   - HashMap<String, HashSet<usize>>  -> indice invertido
//   - BTreeMap<usize, String>          -> documentos ordenados por id
//   - HashSet<usize>                   -> operaciones de conjunto (AND, OR)
//   - Vec<(usize, usize)>              -> ranking de resultados

use std::collections::{BTreeMap, HashMap, HashSet};

// ── ESTRUCTURAS ───────────────────────────────────────────────

struct MotorBusqueda {
    // documentos: id -> contenido original
    // BTreeMap para tenerlos siempre ordenados por id
    documentos: BTreeMap<usize, String>,

    // indice invertido: palabra -> set de ids de documentos que la contienen
    // HashMap porque solo nos importa buscar rapido, no el orden
    indice: HashMap<String, HashSet<usize>>,

    // contador para asignar ids
    siguiente_id: usize,
}

impl MotorBusqueda {
    fn nuevo() -> Self {
        MotorBusqueda {
            documentos: BTreeMap::new(),
            indice: HashMap::new(),
            siguiente_id: 0,
        }
    }

    // agregar un documento al indice
    fn agregar(&mut self, texto: &str) -> usize {
        let id = self.siguiente_id;
        self.siguiente_id += 1;

        // guardar el documento original
        self.documentos.insert(id, texto.to_string());

        // tokenizar: separar en palabras, minusculas, sin signos
        let palabras = tokenizar(texto);

        // por cada palabra, agregar este id al set del indice
        for palabra in palabras {
            self.indice
                .entry(palabra)               // si no existe la entrada, la crea
                .or_insert_with(HashSet::new) // con un HashSet vacio
                .insert(id);                  // agrega el id
        }

        id
    }

    // buscar documentos que contengan UNA palabra
    fn buscar(&self, palabra: &str) -> HashSet<usize> {
        let palabra = limpiar(palabra);
        self.indice
            .get(&palabra)
            .cloned()
            .unwrap_or_default()
    }

    // AND: documentos que contienen TODAS las palabras
    fn buscar_and(&self, palabras: &[&str]) -> HashSet<usize> {
        let mut iter = palabras.iter();

        let primero = match iter.next() {
            Some(p) => self.buscar(p),
            None    => return HashSet::new(),
        };

        // intersectar con los sets de las demas palabras
        iter.fold(primero, |acc, palabra| {
            let set = self.buscar(palabra);
            acc.intersection(&set).cloned().collect()
        })
    }

    // OR: documentos que contienen AL MENOS UNA palabra
    fn buscar_or(&self, palabras: &[&str]) -> HashSet<usize> {
        palabras.iter().fold(HashSet::new(), |acc, palabra| {
            let set = self.buscar(palabra);
            acc.union(&set).cloned().collect()
        })
    }

    // rankear resultados por relevancia
    // relevancia = cuantas palabras del query aparecen en el documento
    fn rankear(&self, ids: &HashSet<usize>, query: &[&str]) -> Vec<(usize, usize)> {
        let mut resultados: Vec<(usize, usize)> = ids
            .iter()
            .map(|&id| {
                let relevancia = query
                    .iter()
                    .filter(|&&palabra| self.buscar(palabra).contains(&id))
                    .count();
                (id, relevancia)
            })
            .collect();

        resultados.sort_by(|a, b| b.1.cmp(&a.1));
        resultados
    }

    // mostrar resultados rankeados
    fn mostrar_resultados(&self, ids: &HashSet<usize>, query: &[&str]) {
        if ids.is_empty() {
            println!("  no se encontraron resultados");
            return;
        }

        let rankeados = self.rankear(ids, query);

        for (id, relevancia) in rankeados {
            if let Some(doc) = self.documentos.get(&id) {
                let preview = if doc.len() > 60 {
                    format!("{}...", &doc[..60])
                } else {
                    doc.clone()
                };
                println!("  [doc {}] relevancia: {} | {}", id, relevancia, preview);
            }
        }
    }

    // mostrar el indice completo
    fn mostrar_indice(&self) {
        println!("--- indice invertido ---");
        let mut palabras: Vec<&String> = self.indice.keys().collect();
        palabras.sort();

        for palabra in palabras {
            let ids: Vec<usize> = {
                let mut v: Vec<usize> = self.indice[palabra].iter().cloned().collect();
                v.sort();
                v
            };
            println!("  '{}' -> docs {:?}", palabra, ids);
        }
    }
}

// ── HELPERS ───────────────────────────────────────────────────

fn limpiar(palabra: &str) -> String {
    palabra
        .to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric())
        .collect()
}

// tokeniza un texto en palabras limpias sin duplicados
fn tokenizar(texto: &str) -> HashSet<String> {
    texto
        .split_whitespace()
        .map(limpiar)
        .filter(|p| !p.is_empty() && p.len() > 2) // ignorar palabras muy cortas
        .collect() // HashSet elimina duplicados automaticamente
}

// ── MAIN ──────────────────────────────────────────────────────

fn main() {
    let mut motor = MotorBusqueda::nuevo();

    println!("--- agregando documentos ---");
    let docs = vec![
        "Rust es un lenguaje de programacion rapido y seguro",
        "Python es popular para machine learning e inteligencia artificial",
        "Rust tiene un sistema de tipos muy poderoso y seguro",
        "El machine learning usa muchos algoritmos matematicos",
        "La inteligencia artificial esta cambiando el mundo tecnologico",
        "Rust y Python son lenguajes modernos muy populares hoy",
        "Los algoritmos de busqueda son fundamentales en programacion",
        "El sistema de ownership de Rust previene errores de memoria",
    ];

    for doc in &docs {
        let id = motor.agregar(doc);
        println!("  [doc {}] {}", id, doc);
    }

    println!();
    motor.mostrar_indice();

    // busquedas simples
    println!("\n--- busqueda simple: 'rust' ---");
    let r = motor.buscar("rust");
    motor.mostrar_resultados(&r, &["rust"]);

    println!("\n--- busqueda simple: 'machine' ---");
    let r = motor.buscar("machine");
    motor.mostrar_resultados(&r, &["machine"]);

    // AND
    println!("\n--- AND: 'rust' AND 'seguro' ---");
    let r = motor.buscar_and(&["rust", "seguro"]);
    motor.mostrar_resultados(&r, &["rust", "seguro"]);

    println!("\n--- AND: 'rust' AND 'python' ---");
    let r = motor.buscar_and(&["rust", "python"]);
    motor.mostrar_resultados(&r, &["rust", "python"]);

    // OR
    println!("\n--- OR: 'rust' OR 'python' ---");
    let r = motor.buscar_or(&["rust", "python"]);
    motor.mostrar_resultados(&r, &["rust", "python"]);

    // query con ranking
    println!("\n--- ranking: 'rust' OR 'seguro' OR 'tipos' ---");
    let query = ["rust", "seguro", "tipos"];
    let r = motor.buscar_or(&query);
    motor.mostrar_resultados(&r, &query);

    // palabra que no existe
    println!("\n--- busqueda: 'javascript' ---");
    let r = motor.buscar("javascript");
    motor.mostrar_resultados(&r, &["javascript"]);

    // stats usando BTreeMap (documentos ordenados)
    println!("\n--- stats ---");
    println!("total documentos: {}", motor.documentos.len());
    println!("palabras indexadas: {}", motor.indice.len());

    let mas_comun = motor.indice.iter().max_by_key(|(_, ids)| ids.len());
    if let Some((palabra, ids)) = mas_comun {
        println!("palabra mas comun: '{}' aparece en {} docs", palabra, ids.len());
    }
}
