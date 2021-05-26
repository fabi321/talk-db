#[derive(Debug)]
pub struct Entry {
    pub name: String,
    pub time: chrono::NaiveDateTime,
    pub participants: Vec<Person>,
    pub url: String,
}

#[derive(Debug)]
pub struct Person {
    pub name: String,
    pub title: Option<String>,
    pub party: Option<String>,
    pub biografie: Option<String>,
    pub image_url: Option<String>,
}

struct Gid {
    gid: Option<i64>,
}

struct Seid {
    seid: Option<i64>,
}

type Transaction<'a> = sqlx::Transaction<'a, sqlx::Sqlite>;

impl Entry {
    pub async fn write_to_db(self, mut transaction: Transaction<'_>, show: i64) -> Transaction<'_> {
        let mut gaeste = Vec::new();
        for gast in self.participants {
            sqlx::query_file!(
                "src/insert_gast.sql",
                gast.name,
                gast.title,
                gast.party,
                gast.biografie,
                gast.image_url
            ).execute(&mut transaction).await.unwrap();
            let gid = sqlx::query_file_as!(
                Gid,
                "src/select_gast.sql",
                gast.name
            ).fetch_one(&mut transaction).await.unwrap();
            gaeste.push(gid.gid.unwrap());
        }
        let seid = sqlx::query_file_as!(
            Seid,
            "src/insert_sendungen.sql",
            show,
            self.name,
            self.url,
            self.time
        ).fetch_one(&mut transaction).await.ok();
        if let Some(seid) = seid {
            let seid = seid.seid.unwrap();
            for gid in gaeste {
                sqlx::query_file!(
                    "src/insert_gastsendung.sql",
                    seid,
                    gid
                ).execute(&mut transaction).await.unwrap();
            }
        }
        transaction
    }
}

