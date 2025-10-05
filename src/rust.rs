use syn::visit_mut::{self, VisitMut};

#[allow(dead_code)]
pub static OPTION_DOC: (&str, &str) = (
    "--strip-rust-tests",
    "Strip test modules and test functions from .rs files.",
);

pub fn strip_tests(contents: &str) -> String {
    // Try to parse as a Rust file
    let Ok(mut syntax_tree) = syn::parse_file(contents) else {
        // If parsing fails, return original content
        return contents.to_string();
    };

    // Visit and remove test items
    let mut visitor = TestStripper;
    visitor.visit_file_mut(&mut syntax_tree);

    // Format the output nicely
    prettyplease::unparse(&syntax_tree)
}

struct TestStripper;

impl VisitMut for TestStripper {
    fn visit_file_mut(&mut self, file: &mut syn::File) {
        // Remove test items from the file
        file.items.retain(|item| !should_remove_item(item));

        // Continue visiting nested items
        visit_mut::visit_file_mut(self, file);
    }

    fn visit_item_mod_mut(&mut self, module: &mut syn::ItemMod) {
        // Check if the module itself should be removed (e.g., #[cfg(test)])
        if has_test_cfg(&module.attrs) {
            // Clear the module content
            module.content = None;
            return;
        }

        // Visit the module's items if it has content
        if let Some((_, items)) = &mut module.content {
            items.retain(|item| !should_remove_item(item));
            for item in items.iter_mut() {
                self.visit_item_mut(item);
            }
        }
    }
}

fn should_remove_item(item: &syn::Item) -> bool {
    match item {
        syn::Item::Fn(func) => has_test_attr(&func.attrs),
        syn::Item::Mod(module) => has_test_cfg(&module.attrs),
        _ => false,
    }
}

fn has_test_attr(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| {
        attr.path()
            .segments
            .last()
            .map(|seg| seg.ident == "test")
            .unwrap_or(false)
    })
}

fn has_test_cfg(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| {
        // Check for #[cfg(test)]
        if attr.path().segments.last().map(|seg| seg.ident == "cfg").unwrap_or(false) {
            if let syn::Meta::List(meta_list) = &attr.meta {
                return meta_list.tokens.to_string().contains("test");
            }
        }
        false
    })
}
