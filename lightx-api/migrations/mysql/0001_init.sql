-- Dialect : MySQL

CREATE TABLE roles (
    id INT AUTO_INCREMENT PRIMARY KEY,
    name VARCHAR(50) NOT NULL
);

CREATE TABLE users (
    id INT AUTO_INCREMENT PRIMARY KEY,
    role_id INT NOT NULL,
    email VARCHAR(255) NOT NULL UNIQUE,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    FOREIGN KEY(role_id) REFERENCES roles(id)
);

CREATE INDEX idx_active_users ON users(is_active);

CREATE TABLE user_preferences (
    user_id INT PRIMARY KEY,
    theme VARCHAR(50),
    FOREIGN KEY(user_id) REFERENCES users(id)
);

CREATE TABLE groups (
    id INT AUTO_INCREMENT PRIMARY KEY,
    name VARCHAR(100) NOT NULL UNIQUE
);

CREATE TABLE user_groups (
    user_id INT NOT NULL,
    group_id INT NOT NULL,
    PRIMARY KEY (user_id, group_id),
    FOREIGN KEY(user_id) REFERENCES users(id),
    FOREIGN KEY(group_id) REFERENCES groups(id)
);

CREATE TABLE categories (
    id INT AUTO_INCREMENT PRIMARY KEY,
    parent_id INT,
    name VARCHAR(100) NOT NULL,
    FOREIGN KEY(parent_id) REFERENCES categories(id)
);

CREATE TABLE all_types_demo (
    id INT AUTO_INCREMENT PRIMARY KEY,
    
    -- Strings
    string_col VARCHAR(255) NOT NULL,
    text_col TEXT,
    
    -- Numbers
    int_col INT NOT NULL,
    bigint_col BIGINT,
    float_col FLOAT,
    double_col DOUBLE,
    decimal_col DECIMAL(10, 2),
    
    -- Booleans
    bool_col BOOLEAN NOT NULL DEFAULT FALSE,
    
    -- Dates and Times
    date_col DATE,
    time_col TIME,
    datetime_col DATETIME,
    timestamp_col TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    
    -- Binary and others
    binary_col BLOB,
    json_col JSON
);
