mod client;
mod commands;
mod output;

use clap::{Parser, Subcommand};
use client::Client;
use output::Format;

#[derive(Parser)]
#[command(
    name = "openquality",
    version,
    about = "OpenQuality CLI v2 — data quality and observability"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Output as JSON
    #[arg(global = true, long, short = 'j', conflicts_with = "plain")]
    json: bool,

    /// Plain text output
    #[arg(global = true, long, short = 'p')]
    plain: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Health check
    Health,

    /// Workspace management
    #[command(subcommand)]
    Workspaces(WorkspaceCommands),

    /// Monitor management
    #[command(subcommand)]
    Monitors(MonitorCommands),

    /// Incident management
    #[command(subcommand)]
    Incidents(IncidentCommands),

    /// Data source management
    #[command(subcommand)]
    DataSources(DataSourceCommands),

    /// Integrations (dbt, GE, lineage)
    #[command(subcommand)]
    Integrations(IntegrationCommands),
}

#[derive(Subcommand)]
enum WorkspaceCommands {
    /// List all workspaces
    List,
    /// Create a new workspace
    Create { name: String, slug: String },
}

#[derive(Subcommand)]
enum MonitorCommands {
    /// List monitors in a workspace
    List { workspace_id: String },
    /// Create a new monitor
    Create {
        workspace_id: String,
        name: String,
        #[arg(long)]
        r#type: String,
        #[arg(long)]
        table: String,
        #[arg(long)]
        cron: Option<String>,
    },
    /// Delete a monitor
    Delete { id: String },
    /// Run a monitor
    Run { id: String },
}

#[derive(Subcommand)]
enum IncidentCommands {
    /// List all incidents
    List,
    /// Show incident details
    Show { id: String },
    /// Acknowledge an incident
    Acknowledge { id: String },
    /// Resolve an incident
    Resolve { id: String },
}

#[derive(Subcommand)]
enum DataSourceCommands {
    /// List data sources in a workspace
    List { workspace_id: String },
    /// Create a new data source
    Create {
        workspace_id: String,
        name: String,
        #[arg(long)]
        connector: String,
        #[arg(long)]
        connection_string: String,
    },
    /// Delete a data source
    Delete { id: String },
}

#[derive(Subcommand)]
enum IntegrationCommands {
    /// Parse a dbt manifest.json file
    Dbt { manifest: String },
    /// Translate a Great Expectations suite JSON to OpenQuality
    Ge { suite: String },
    /// Parse a SQL string for column lineage
    LineageSql { sql: String },
    /// Build a full lineage graph from SQL files
    LineageGraph { files: Vec<String> },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let format = Format::from_args(cli.json, cli.plain);
    let client = Client::from_env();

    match cli.command {
        Commands::Health => commands::health::run(&client, &format).await,

        Commands::Workspaces(cmd) => match cmd {
            WorkspaceCommands::List => commands::workspaces::list(&client, &format).await,
            WorkspaceCommands::Create { name, slug } => {
                commands::workspaces::create(&client, &name, &slug, &format).await
            }
        },

        Commands::Monitors(cmd) => match cmd {
            MonitorCommands::List { workspace_id } => {
                commands::monitors::list(&client, &workspace_id, &format).await
            }
            MonitorCommands::Create {
                workspace_id,
                name,
                r#type,
                table,
                cron,
            } => {
                commands::monitors::create(
                    &client,
                    &workspace_id,
                    &name,
                    &r#type,
                    &table,
                    cron.as_deref(),
                    &format,
                )
                .await
            }
            MonitorCommands::Delete { id } => {
                commands::monitors::delete(&client, &id, &format).await
            }
            MonitorCommands::Run { id } => commands::monitors::run(&client, &id, &format).await,
        },

        Commands::Incidents(cmd) => match cmd {
            IncidentCommands::List => commands::incidents::list(&client, &format).await,
            IncidentCommands::Show { id } => commands::incidents::get(&client, &id, &format).await,
            IncidentCommands::Acknowledge { id } => {
                commands::incidents::acknowledge(&client, &id, &format).await
            }
            IncidentCommands::Resolve { id } => {
                commands::incidents::resolve(&client, &id, &format).await
            }
        },

        Commands::DataSources(cmd) => match cmd {
            DataSourceCommands::List { workspace_id } => {
                commands::data_sources::list(&client, &workspace_id, &format).await
            }
            DataSourceCommands::Create {
                workspace_id,
                name,
                connector,
                connection_string,
            } => {
                commands::data_sources::create(
                    &client,
                    &workspace_id,
                    &name,
                    &connector,
                    &connection_string,
                    &format,
                )
                .await
            }
            DataSourceCommands::Delete { id } => {
                commands::data_sources::delete(&client, &id, &format).await
            }
        },

        Commands::Integrations(cmd) => match cmd {
            IntegrationCommands::Dbt { manifest } => {
                commands::integrations::dbt_parse_manifest(&client, &manifest, &format).await
            }
            IntegrationCommands::Ge { suite } => {
                commands::integrations::ge_translate(&client, &suite, &format).await
            }
            IntegrationCommands::LineageSql { sql } => {
                commands::integrations::lineage_parse_sql(&client, &sql, &format).await
            }
            IntegrationCommands::LineageGraph { files } => {
                commands::integrations::lineage_build_graph(&client, &files, &format).await
            }
        },
    }
}
