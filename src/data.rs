use crate::AppError;
use crate::SourceDetails;
use sqlx::{Pool, Postgres};


pub async fn get_all_ids(pool: &Pool<Postgres>) -> Result<Vec<i32>, AppError> {

    let sql = r#"select id from sf.source_parameters
    where id > 100115 and id not in (100159, 101405, 101940, 110426)
    order by preference_rating"#;

    let ids = sqlx::query_scalar(sql).fetch_all(pool).await?;
    Ok(ids)
}


pub async fn get_source_details(source_id: i32, pool: &Pool<Postgres>) -> Result<SourceDetails, AppError> {

    let sql = r#"select id, database_name, local_folder, 
       local_files_grouped, grouping_range_by_id from sf.source_parameters
       where id = $1"#;

    let src_details: SourceDetails = sqlx::query_as(sql).bind(source_id).fetch_one(pool).await?;
    Ok(src_details)

}



       