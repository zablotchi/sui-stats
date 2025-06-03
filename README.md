# Sui Sizes

A Rust-based indexer that analyzes and tracks size metrics of data in the Sui blockchain. This tool processes Sui checkpoints and stores detailed size analytics in a PostgreSQL database.

## What It Does

Sui Sizes monitors and records various size metrics for each checkpoint in the Sui blockchain:

### Metrics Tracked
- **Checkpoint-level**: Summary, signatures, and contents sizes
- **Transaction-level**: Count, data size, effects size, and events size  
- **Object-level**: Count and total data size

### Database Schema
The tool creates a `sizes` table with the following structure:
```sql
CREATE TABLE sizes (
    cp_sequence_number          BIGINT PRIMARY KEY,
    cp_summary_bytes            BIGINT NOT NULL,
    cp_signatures_bytes         BIGINT NOT NULL,
    cp_contents_bytes           BIGINT NOT NULL,
    tx_count                    BIGINT NOT NULL,
    tx_bytes                    BIGINT NOT NULL,
    fx_bytes                    BIGINT NOT NULL,
    ev_bytes                    BIGINT NOT NULL,
    obj_count                   BIGINT NOT NULL,
    obj_bytes                   BIGINT NOT NULL
);
```

## Prerequisites

1. **Rust** (latest stable version)
2. **PostgreSQL** (version 12 or higher)
3. **Diesel CLI** for database migrations

## Setup Instructions

### 1. Install Dependencies

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Diesel CLI with PostgreSQL support
cargo install diesel_cli --no-default-features --features postgres
```

### 2. Set Up PostgreSQL

#### Option A: Local PostgreSQL Installation
```bash
# macOS (using Homebrew)
brew install postgresql
brew services start postgresql

# Create database and user
createdb sui_sizes
psql -d sui_sizes -c "CREATE USER postgres WITH PASSWORD 'postgrespw';"
psql -d sui_sizes -c "GRANT ALL PRIVILEGES ON DATABASE sui_sizes TO postgres;"
```

#### Option B: Docker PostgreSQL
```bash
# Run PostgreSQL in Docker
docker run --name sui-sizes-db \
  -e POSTGRES_DB=sui_sizes \
  -e POSTGRES_USER=postgres \
  -e POSTGRES_PASSWORD=postgrespw \
  -p 5432:5432 \
  -d postgres:15

# Verify it's running
docker ps
```

### 3. Set Up the Database Schema

```bash
# Clone and navigate to the project
cd sui-sizes

# Set the database URL (optional if using defaults)
export DATABASE_URL="postgres://postgres:postgrespw@localhost:5432/sui_sizes"

# Run database migrations
diesel migration run
```

### 4. Build the Project

```bash
# Build the project
cargo build --release
```

## Running the Tool

### Basic Usage (Local Database)
```bash
# Run with default settings (connects to local PostgreSQL)
cargo run --release
```

### Custom Database URL
```bash
# Run with custom database connection
cargo run --release -- --database-url "postgres://user:password@host:port/database"
```

### With Additional Cluster Arguments
```bash
# Run with custom checkpoint range
cargo run --release -- \
  --database-url "postgres://postgres:postgrespw@localhost:5432/sui_sizes" \
  --first-checkpoint 1000 \
  --last-checkpoint 2000
```

### Command Line Options

```bash
# View all available options
cargo run --release -- --help
```

Common options include:
- `--database-url`: PostgreSQL connection string (default: `postgres://postgres:postgrespw@localhost:5432/sui_sizes`)
- `--first-checkpoint`: Starting checkpoint number
- `--last-checkpoint`: Ending checkpoint number
- `--checkpoint-buffer-size`: Number of checkpoints to buffer

## Monitoring Progress

The tool will output logs showing:
- Checkpoint processing progress
- Database insertion status
- Any errors encountered

Example output:
```
Processing checkpoint 12345...
Inserted 1 size record for checkpoint 12345
Processing checkpoint 12346...
```

## Querying the Data

Once the tool is running and processing checkpoints, you can query the data:

```sql
-- Connect to the database
psql -d sui_sizes -U postgres

-- View recent checkpoint sizes
SELECT cp_sequence_number, tx_count, tx_bytes, obj_count 
FROM sizes 
ORDER BY cp_sequence_number DESC 
LIMIT 10;

-- Calculate average transaction size
SELECT AVG(tx_bytes::float / tx_count) as avg_tx_size 
FROM sizes 
WHERE tx_count > 0;

-- Track growth over time
SELECT cp_sequence_number, 
       (cp_summary_bytes + cp_signatures_bytes + cp_contents_bytes + tx_bytes + fx_bytes + ev_bytes + obj_bytes) as total_bytes
FROM sizes 
ORDER BY cp_sequence_number;
```

## Troubleshooting

### Database Connection Issues
```bash
# Test PostgreSQL connection
psql -h localhost -p 5432 -U postgres -d sui_sizes

# Check if PostgreSQL is running
brew services list | grep postgresql  # macOS
docker ps | grep postgres             # Docker
```

### Migration Issues
```bash
# Reset migrations (WARNING: This will drop all data)
diesel migration revert
diesel migration run

# Check migration status
diesel migration list
```

### Build Issues
```bash
# Clean and rebuild
cargo clean
cargo build --release
```

## Use Cases

- **Blockchain Analytics**: Understanding data growth patterns on Sui
- **Performance Monitoring**: Tracking checkpoint and transaction size evolution
- **Resource Planning**: Estimating storage and bandwidth requirements
- **Research**: Analyzing efficiency of different transaction types

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests: `cargo test`
5. Submit a pull request

## License

[Add your license information here] 