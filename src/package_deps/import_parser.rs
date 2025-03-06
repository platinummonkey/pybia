use super::models::ImportInfo;

pub struct ImportParser {
    content: String,
}

impl ImportParser {
    pub fn new(content: String) -> Self {
        Self { content }
    }

    pub fn parse_imports(&mut self) -> Vec<ImportInfo> {
        let mut imports = Vec::new();
        let lines: Vec<&str> = self.content.lines().collect();

        let mut i = 0;
        while i < lines.len() {
            let line = lines[i].trim();
            
            if let Some(stripped) = line.strip_prefix("import ") {
                let imported = stripped.trim();
                // Handle multiple imports separated by commas
                for import_part in imported.split(',') {
                    let import_part = import_part.trim();
                    if !import_part.is_empty() {
                        if let Some(import_info) = self.parse_direct_import(&format!("import {}", import_part)) {
                            imports.push(import_info);
                        }
                    }
                }
            } else if line.starts_with("from ") {
                if let Some(import_info) = self.parse_from_import(line) {
                    imports.push(import_info);
                }
            }
            
            i += 1;
        }

        imports
    }

    fn parse_from_import(&self, statement: &str) -> Option<ImportInfo> {
        let parts: Vec<&str> = statement["from ".len()..].split(" import ").collect();
        if parts.len() != 2 {
            return None;
        }

        let module_path: Vec<String> = parts[0].split('.')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        
        if module_path.is_empty() {
            return None;
        }

        let package_name = module_path[0].clone();
        let imported_names: Vec<String> = parts[1]
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        Some(ImportInfo {
            package_name,
            module_path,
            is_from_import: true,
            imported_names,
        })
    }

    fn parse_direct_import(&self, statement: &str) -> Option<ImportInfo> {
        let imported = statement["import ".len()..].trim();
        
        // Handle single import with possible "as" alias
        let parts: Vec<String> = imported.split(" as ").nth(0)?
            .split('.')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        if !parts.is_empty() {
            let package_name = parts[0].clone();
            return Some(ImportInfo {
                package_name: package_name.clone(),
                module_path: parts,
                is_from_import: false,
                imported_names: vec![package_name],
            });
        }
        
        None
    }
} 