-- Une table classique avec une PK
CREATE TABLE users (
    id BIGINT AUTO_INCREMENT PRIMARY KEY,
    email VARCHAR(255) NOT NULL,
    first_name VARCHAR(100),
    last_name VARCHAR(100) NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    
    -- Index simple (unique) : pour générer get_by_email, delete_by_email
    UNIQUE INDEX idx_email (email),
    
    -- Index multiple : pour générer update_by_last_name_and_first_name
    INDEX idx_name (last_name, first_name)
);

-- Une vue : ne générera que les fonctions de lecture (get, stream, list...)
CREATE VIEW active_users AS 
SELECT id, email, first_name, last_name 
FROM users 
WHERE status = 'active';

-- Une table de relation (sans PK explicite, mais avec des index)
CREATE TABLE user_roles (
    user_id BIGINT NOT NULL,
    role_name VARCHAR(50) NOT NULL,
    assigned_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    
    -- Index pour la recherche rapide
    INDEX idx_user_id (user_id),
    
    -- Index unique sur le couple (souvent utilisé comme pseudo-PK dans une table de liaison)
    UNIQUE INDEX idx_user_role (user_id, role_name)
);

-- 1. Clé primaire composite
CREATE TABLE order_items (
    order_id BIGINT NOT NULL,
    product_id BIGINT NOT NULL,
    quantity INT NOT NULL DEFAULT 1,
    PRIMARY KEY (order_id, product_id)
);

-- 2. Mots-clés réservés en Rust
CREATE TABLE configurations (
    id INT AUTO_INCREMENT PRIMARY KEY,
    `type` VARCHAR(50) NOT NULL,
    `match` VARCHAR(255),
    value TEXT
);

-- 3. Types de données complexes
CREATE TABLE product_metadata (
    id BINARY(16) PRIMARY KEY,
    category ENUM('tech', 'food', 'books') NOT NULL,
    attributes JSON,
    raw_data BLOB
);

-- 4. Clé primaire SANS auto-increment
CREATE TABLE currencies (
    code VARCHAR(3) PRIMARY KEY,
    name VARCHAR(50) NOT NULL
);

-- ==========================================
-- JEU DE DONNÉES DE TEST (DUMP INITIAL)
-- ==========================================

-- 1. Insertion des devises
INSERT INTO currencies (code, name) VALUES 
('EUR', 'Euro'),
('USD', 'US Dollar'),
('GBP', 'British Pound'),
('JPY', 'Japanese Yen');

-- 2. Insertion d'utilisateurs
INSERT INTO users (email, first_name, last_name, status) VALUES 
('alice@example.com', 'Alice', 'Dupont', 'active'),
('bob@example.com', 'Bob', 'Martin', 'active'),
('charlie@example.com', 'Charlie', 'Durand', 'inactive'),
('diana@example.com', 'Diana', 'Prince', 'active');

-- 3. Insertion des rôles (utilisateurs 1 à 4 auto-générés)
INSERT INTO user_roles (user_id, role_name) VALUES 
(1, 'admin'),
(1, 'user'),
(2, 'user'),
(3, 'guest'),
(4, 'manager');

-- 4. Insertion d'éléments de commande (Clé composite)
INSERT INTO order_items (order_id, product_id, quantity) VALUES 
(1001, 501, 2),
(1001, 502, 1),
(1002, 501, 5),
(1003, 503, 10);

-- 5. Insertion de configurations (Mots-clés réservés Rust : type, match)
INSERT INTO configurations (`type`, `match`, value) VALUES 
('theme', 'dark_mode', 'true'),
('language', 'default', 'fr-FR'),
('pagination', 'per_page', '25');

-- 6. Insertion de métadonnées produits (Types complexes : JSON, Enum, Binary)
INSERT INTO product_metadata (id, category, attributes, raw_data) VALUES 
(UNHEX(REPLACE('550e8400-e29b-41d4-a716-446655440000', '-', '')), 'tech', '{"brand": "Apple", "warranty": "2 years"}', 'binary_data_1'),
(UNHEX(REPLACE('123e4567-e89b-12d3-a456-426614174000', '-', '')), 'food', '{"organic": true, "calories": 250}', 'binary_data_2'),
(UNHEX(REPLACE('987e6543-e21b-34d5-b678-426614174111', '-', '')), 'books', '{"pages": 350, "author": "John Doe"}', 'binary_data_3');
