use sqlbinder::context_container::ContextContainer;
use sqlbinder::db::get_connection_pool;

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let mut context = ContextContainer::new()?;
	let database_settings = context.get_database_settings();
	let connection_pool = get_connection_pool(database_settings.as_ref())?;

	context.set_connection_pool(connection_pool);

	println!(
		"sqlbinder initialized for the {} environment",
		context.get_environment_name()
	);

	Ok(())
}
