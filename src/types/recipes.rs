pub struct ShapedRecipe {
    pub pattern: [i32; 9],
    pub width: i32,
    pub height: i32,
    pub result_id: i32,
    pub result_count: i32,
}

pub struct ShapelessRecipe {
    pub ingredients: &'static [i32],
    pub result_id: i32,
    pub result_count: i32,
}

pub enum Recipe {
    Shaped(ShapedRecipe),
    Shapeless(ShapelessRecipe),
}

include!(concat!(env!("OUT_DIR"), "/recipes.rs"));
include!(concat!(env!("OUT_DIR"), "/furnace_recipes.rs"));

pub fn check_2x2(grid: &[Option<i32>; 4]) -> Option<(i32, u8)> {
    for recipe in RECIPES {
        match recipe {
            Recipe::Shapeless(r) => {
                if match_shapeless(grid, r) {
                    return Some((r.result_id, r.result_count as u8));
                }
            }

            Recipe::Shaped(r) => {
                if r.width <= 2 && r.height <= 2 {
                    if match_2x2_shaped(grid, r) {
                        return Some((r.result_id, r.result_count as u8));
                    }
                } else {
                    // todo: 3x3 matching
                }
            }
        }
    }

    None
}

pub fn check_3x3(grid: &[Option<i32>; 9]) -> Option<(i32, u8)> {
    for recipe in RECIPES {
        match recipe {
            Recipe::Shapeless(r) => {
                if match_shapeless(grid, r) {
                    return Some((r.result_id, r.result_count as u8));
                }
            }

            Recipe::Shaped(r) => {
                if r.width <= 3 && r.height <= 3 {
                    if match_3x3_shaped(grid, r) {
                        return Some((r.result_id, r.result_count as u8));
                    }
                }
            }
        }
    }

    None
}

fn match_2x2_shaped(grid: &[Option<i32>; 4], recipe: &ShapedRecipe) -> bool {
    for r in 0..=(2usize.saturating_sub(recipe.height as usize)) {
        for c in 0..=(2usize.saturating_sub(recipe.width as usize)) {
            if (0..2).all(|i| {
                (0..2).all(|j| {
                    let grid_item = grid[i * 2 + j].unwrap_or(0);

                    let recipe_row = i as i32 - r as i32;
                    let recipe_col = j as i32 - c as i32;

                    if recipe_row < 0
                        || recipe_row >= recipe.height
                        || recipe_col < 0
                        || recipe_col >= recipe.width
                    {
                        return grid_item == 0;
                    }

                    let recipe_item = recipe.pattern[recipe_row as usize * 3 + recipe_col as usize];

                    grid_item == recipe_item
                })
            }) {
                return true;
            }
        }
    }

    false
}

fn match_3x3_shaped(grid: &[Option<i32>; 9], recipe: &ShapedRecipe) -> bool {
    for r in 0..=(3usize.saturating_sub(recipe.height as usize)) {
        for c in 0..=(3usize.saturating_sub(recipe.width as usize)) {
            if (0..3).all(|i| {
                (0..3).all(|j| {
                    let grid_item = grid[i * 3 + j].unwrap_or(0);

                    let recipe_row = i as i32 - r as i32;
                    let recipe_col = j as i32 - c as i32;

                    if recipe_row < 0
                        || recipe_row >= recipe.height
                        || recipe_col < 0
                        || recipe_col >= recipe.width
                    {
                        return grid_item == 0;
                    }

                    let recipe_item = recipe.pattern[recipe_row as usize * 3 + recipe_col as usize];

                    grid_item == recipe_item
                })
            }) {
                return true;
            }
        }
    }

    false
}

fn match_shapeless(grid: &[Option<i32>], recipe: &ShapelessRecipe) -> bool {
    let grid_items = grid.iter().filter_map(|&item| item).collect::<Vec<_>>();

    if grid_items.len() != recipe.ingredients.len() {
        return false;
    }

    let mut ingredients = recipe.ingredients.to_vec();
    let mut grid_items: Vec<i32> = grid.iter().filter_map(|&item| item).collect();

    ingredients.sort_unstable();
    grid_items.sort_unstable();

    ingredients == grid_items
}
