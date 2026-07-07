use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "vmr", about = "Version Manager", version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Show version info
    #[command(visible_alias = "v")]
    Version,

    /// Install and switch SDK version (format: pluginname@version)
    #[command(visible_aliases = ["u", "h"])]
    Use {
        /// SDK and version in format sdkname@version
        version_info: String,
        /// Use version only for current session
        #[arg(short, long)]
        session_only: bool,
        /// Lock version for current project
        #[arg(short, long)]
        lock_version: bool,
        /// Enable locked version for current project
        #[arg(short = 'E', long)]
        enable_locked_version: bool,
        /// Install by conda
        #[arg(short = 'c', long)]
        install_by_conda: bool,
    },

    /// Search available versions for an SDK
    #[command(visible_alias = "s")]
    Search {
        /// SDK/plugin name to search
        sdk_name: String,
        /// Search via conda
        #[arg(short = 'c', long)]
        search_by_conda: bool,
    },

    /// Show available SDKs
    #[command(visible_alias = "S")]
    Show,

    /// Show installed versions for an SDK
    #[command(visible_alias = "l")]
    Local {
        /// SDK name
        sdk_name: String,
    },

    /// Show installed SDKs
    #[command(visible_alias = "in")]
    InstalledSdks,

    /// Show installed SDK information
    #[command(visible_alias = "ii")]
    InstalledInfo,

    /// Uninstall SDK versions (plugin@version or plugin@all)
    #[command(visible_aliases = ["uni", "r"])]
    Uninstall {
        /// Plugin and version: pluginname@version or pluginname@all
        version_info: String,
    },

    /// Install VMR itself
    #[command(visible_aliases = ["i", "is"])]
    InstallSelf,

    /// Uninstall VMR
    #[command(visible_alias = "Uins")]
    UninstallSelf,

    /// Update plugins
    #[command(visible_alias = "up")]
    UpdatePlugins,
}

pub fn run() {
    let cli = Cli::parse();

    match &cli.command {
        None => {
            // Default: show TUI (placeholder for now)
            println!("VMR - Version Manager");
            println!("Run 'vmr --help' for usage.");
        }
        Some(cmd) => match cmd {
            Commands::Version => {
                println!("vmr {}", env!("CARGO_PKG_VERSION"));
            }
            Commands::Show => {
                println!("Showing available SDKs...");
            }
            Commands::Search {
                sdk_name,
                search_by_conda,
            } => {
                println!("Searching for {} (conda: {})...", sdk_name, search_by_conda);
            }
            Commands::Local { sdk_name } => {
                println!("Local versions for {}:", sdk_name);
            }
            Commands::InstalledSdks => {
                println!("Installed SDKs:");
            }
            Commands::InstalledInfo => {
                println!("Installed SDK info:");
            }
            Commands::Use {
                version_info,
                session_only,
                lock_version,
                enable_locked_version,
                install_by_conda,
            } => {
                if *enable_locked_version {
                    println!("Enabling locked version...");
                } else if let Some((sdk, ver)) = version_info.split_once('@') {
                    println!(
                        "Installing {}@{} (session={}, lock={}, conda={})",
                        sdk, ver, session_only, lock_version, install_by_conda
                    );
                } else {
                    eprintln!("Usage: vmr use sdkname@version");
                }
            }
            Commands::Uninstall { version_info } => {
                if let Some((sdk, ver)) = version_info.split_once('@') {
                    println!("Uninstalling {}@{}...", sdk, ver);
                } else {
                    eprintln!("Usage: vmr uninstall sdkname@version or sdkname@all");
                }
            }
            Commands::InstallSelf => {
                vmr_self::install::install_self();
            }
            Commands::UninstallSelf => {
                vmr_self::old_versions::remove_current_version();
            }
            Commands::UpdatePlugins => {
                println!("Updating plugins...");
            }
        },
    }
}
