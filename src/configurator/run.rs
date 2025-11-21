use async_sqlite::Pool;
use log::{debug, info};

use crate::{
    configurator::build::Plan,
    db::{events::Events, years::Years},
};

pub async fn run(plan: Plan, pool: &Pool) -> Result<(), async_sqlite::Error> {
    info!("Implementing Plan");
    Events::delete_all(&pool).await.unwrap();
    Years::delete_all(&pool).await.unwrap();
    for year in plan.year_plans.iter() {
        debug!("Inserting Planned Year {}", year.id);
        let mut year_struct = Years::new(year.id.clone(), year.name.clone())
            .insert(&pool)
            .await?;
        for event in year.events.iter() {
            debug!("Inserting Planned Event {}", event.id);
            year_struct = year_struct
                .new_event(
                    &pool,
                    event.clone().id,
                    event.clone().name,
                    event.clone().gender_id,
                    event.clone().filter_key,
                    event.clone().scores,
                )
                .await?
        }
    }
    Ok(())
}
