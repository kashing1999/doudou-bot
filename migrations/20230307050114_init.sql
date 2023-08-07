-- Add migration script here
CREATE TABLE products (
    id     int          NOT NULL AUTO_INCREMENT,
    vendor varchar(64)  NOT NULL,
    url    varchar(255) NOT NULL,
    time   TIMESTAMP    DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    PRIMARY KEY (id)
);
