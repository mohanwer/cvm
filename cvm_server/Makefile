DB_NAME = client_version_manager
DB_USER = $(or $(POSTGRES_USER),postgres)
PSQL = psql -U $(DB_USER)

.PHONY: recreate_db
initialize_databases: drop_database setup_db drop_test_db setup_test_db seed_database

.PHONY: create_database
create_database:
	echo "Checking if database '$(DB_NAME)' exists..."
	@$(PSQL) -d postgres -tAc "SELECT 1 FROM pg_database WHERE datname = '$(DB_NAME)'" | grep -q 1 || \
	($(PSQL) -d postgres -c 'CREATE DATABASE $(DB_NAME);' && echo "Database '$(DB_NAME)' created.")

.PHONY: initialize_tables
initialize_tables:
	@$(PSQL) -d $(DB_NAME) -f db/init_tables.sql

.PHONY: setup_db
setup_db: create_database initialize_tables

.PHONY: drop_database
drop_database:
	@echo "Dropping database '$(DB_NAME)' if it exists..."
	@$(PSQL) -d postgres -tAc "SELECT 1 FROM pg_database WHERE datname = '$(DB_NAME)'" | grep -q 1 && \
	($(PSQL) -d postgres -c 'DROP DATABASE $(DB_NAME);' && echo "Database '$(DB_NAME)' dropped.") || \
	echo "Database '$(DB_NAME)' does not exist."

.PHONY: create_test_database
create_test_database:
	echo "Checking if test database '$(DB_NAME)_test' exists..."
	@$(PSQL) -d postgres -tAc "SELECT 1 FROM pg_database WHERE datname = '$(DB_NAME)_test'" | grep -q 1 || \
	($(PSQL) -d postgres -c 'CREATE DATABASE $(DB_NAME)_test;' && echo "Test database '$(DB_NAME)_test' created.")

.PHONY: initialize_test_tables
initialize_test_tables:
	@$(PSQL) -d $(DB_NAME)_test -f db/init_tables.sql

.PHONY: setup_test_db
setup_test_db: create_test_database initialize_test_tables

.PHONY: drop_test_db
drop_test_db:
	@echo "Dropping test database '$(DB_NAME)_test' if it exists..."
	@$(PSQL) -d postgres -tAc "SELECT 1 FROM pg_database WHERE datname = '$(DB_NAME)_test'" | grep -q 1 && \
	($(PSQL) -d postgres -c 'DROP DATABASE $(DB_NAME)_test;' && echo "Test database '$(DB_NAME)_test' dropped.") || \
	echo "Test database '$(DB_NAME)_test' does not exist."
	

.PHONY: seed_database
seed_database:
	@$(PSQL) -d $(DB_NAME) -f db/seed.sql