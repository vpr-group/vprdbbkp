use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// AWS region (overrides config)
    #[arg(long, env = "S3_REGION")]
    pub region: Option<String>,

    /// S3 bucket name
    #[arg(long, env = "S3_BUCKET")]
    pub bucket: String,

    /// S3 prefix (folder path)
    #[arg(long, env = "S3_PREFIX", default_value = "backups")]
    pub prefix: String,

    /// Custom S3 endpoint URL for third-party S3-compatible services
    #[arg(long, env = "S3_ENDPOINT")]
    pub endpoint: Option<String>,

    /// S3 access key ID (overrides AWS_ACCESS_KEY_ID)
    #[arg(long, env = "S3_ACCESS_KEY_ID")]
    pub access_key: Option<String>,

    /// S3 secret access key (overrides AWS_SECRET_ACCESS_KEY)
    #[arg(long, env = "S3_SECRET_ACCESS_KEY")]
    pub secret_key: Option<String>,

    /// Skip TLS verification for S3 endpoint (not recommended for production)
    #[arg(long)]
    pub no_verify_ssl: bool,

    /// Verbose output
    #[arg(short, long)]
    pub verbose: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Backup a PostgreSQL database
    Postgres {
        /// Database name
        #[arg(short, long)]
        database: String,

        /// Host name
        #[arg(long, short = 'H', default_value = "localhost")]
        host: String,

        /// Port number
        #[arg(short, long, default_value = "5432")]
        port: u16,

        /// Username
        #[arg(short, long)]
        username: String,

        /// Password (prefer using PGPASSWORD env var)
        #[arg(long, env = "PGPASSWORD")]
        password: Option<String>,

        /// Compression level (0-9)
        #[arg(short, long, default_value = "6")]
        compression: u8,

        /// Force using Docker even if local pg_dump is compatible
        #[arg(long)]
        force_docker: bool,
    },

    /// Backup a MySQL/MariaDB database
    Mysql {
        /// Database name
        #[arg(short, long)]
        database: String,

        /// Host name
        #[arg(short, long, default_value = "localhost")]
        host: String,

        /// Port number
        #[arg(short, long, default_value = "3306")]
        port: u16,

        /// Username
        #[arg(short, long)]
        username: String,

        /// Password (prefer using MYSQL_PWD env var)
        #[arg(long, env = "MYSQL_PWD")]
        password: Option<String>,

        /// Compression level (0-9)
        #[arg(short, long, default_value = "6")]
        compression: u8,
    },
}
