use super::models::ImportInfo;

pub struct ImportParser {
    content: String,
    pos: usize,
}

impl ImportParser {
    pub fn new(content: String) -> Self {
        Self { content, pos: 0 }
    }

    pub fn parse_imports(&mut self) -> Vec<ImportInfo> {
        let mut imports = Vec::new();
        let lines: Vec<&str> = self.content.lines().collect();

        let mut i = 0;
        while i < lines.len() {
            let line = lines[i].trim();
            
            if line.starts_with("import ") || line.starts_with("from ") {
                let mut full_statement = line.to_string();
                
                // Handle multi-line imports
                while line.ends_with("\\") || (line.contains("(") && !line.contains(")")) {
                    i += 1;
                    if i < lines.len() {
                        full_statement.push_str(lines[i].trim());
                    }
                }

                if let Some(import_info) = self.parse_import_statement(&full_statement) {
                    imports.push(import_info);
                }
            }
            i += 1;
        }

        imports
    }

    fn parse_import_statement(&self, statement: &str) -> Option<ImportInfo> {
        if statement.starts_with("from ") {
            self.parse_from_import(statement)
        } else {
            self.parse_direct_import(statement)
        }
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
        let imports: Vec<&str> = imported.split(',').map(|s| s.trim()).collect();
        
        let mut result = Vec::new();
        for import in imports {
            let parts: Vec<String> = import.split(" as ").nth(0)?
                .split('.')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();

            if !parts.is_empty() {
                let package_name = parts[0].clone();
                result.push(ImportInfo {
                    package_name: package_name.clone(),
                    module_path: parts,
                    is_from_import: false,
                    imported_names: vec![package_name],
                });
            }
        }
        
        result.into_iter().next()
    }
} 