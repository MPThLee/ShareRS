use rand::Rng;

pub enum DBType {
    File,
    Url,
}

pub async fn name_gen<'a, E>(
    length: usize,
    target: DBType,
    ext: Option<String>,
    pool: E,
) -> anyhow::Result<String>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
{
    let mut name: String;
    for _ in 0..10 {
        name = generate_random_string(length);
        if let Some(e) = &ext {
            name = format!("{}.{}", name, e)
        }

        // Sqlx's query! macro doens't like dynamic table usage...
        let query = match target {
            DBType::File => sqlx::query("SELECT * FROM files f WHERE f.name = $1"),
            DBType::Url => sqlx::query("SELECT * FROM url u WHERE u.name = $1"),
        }
        .bind(&name);

        if (query.fetch_optional(pool).await?).is_none() {
            return Ok(name);
        };
    }

    Err(anyhow::anyhow!("Can't generate new unique name!"))
}

fn generate_random_string(length: usize) -> String {
    let mut rng = rand::thread_rng();
    let chars: Vec<char> = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"
        .chars()
        .collect();

    let random_string: String = (0..length)
        .map(|_| chars[rng.gen_range(0..chars.len())])
        .collect();

    random_string
}
