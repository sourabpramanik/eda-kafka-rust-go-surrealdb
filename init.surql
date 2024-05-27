DEFINE TABLE inventory SCHEMAFULL;

-- Define some fields.
DEFINE FIELD id ON TABLE inventory TYPE record<inventory>;
DEFINE FIELD name ON TABLE inventory TYPE string;
DEFINE FIELD price ON TABLE inventory TYPE number;
DEFINE FIELD units ON TABLE inventory TYPE number;

INSERT INTO inventory 
  [
    { "name": "Nvidia Graphics RTX 3060", "price": 140, "units": 200 },
    { "name": "Logitech Wireless Mouse", "price": 25, "units": 300 },
    { "name": "Corsair Mechanical Keyboard", "price": 100, "units": 150 },
    { "name": "Samsung 1TB SSD", "price": 120, "units": 180 },
    { "name": "LG 27-inch Monitor", "price": 220, "units": 90 },
    { "name": "HyperX Cloud II Headset", "price": 90, "units": 200 },
    { "name": "Razer Gaming Mousepad", "price": 30, "units": 400 },
    { "name": "Intel Core i7 CPU", "price": 300, "units": 80 },
    { "name": "Asus Prime Motherboard", "price": 150, "units": 120 },
    { "name": "Kingston 16GB RAM", "price": 70, "units": 250 },
    { "name": "Seagate 2TB Hard Drive", "price": 60, "units": 230 },
    { "name": "Cooler Master Power Supply", "price": 85, "units": 100 },
    { "name": "TP-Link WiFi Adapter", "price": 20, "units": 300 },
    { "name": "Dell 24-inch Monitor", "price": 180, "units": 110 },
    { "name": "Logitech USB Webcam", "price": 50, "units": 170 },
    { "name": "Razer Mechanical Keyboard", "price": 130, "units": 140 },
    { "name": "WD 500GB SSD", "price": 60, "units": 220 },
    { "name": "Anker USB-C Hub", "price": 40, "units": 190 },
    { "name": "MSI GeForce GTX 1650", "price": 150, "units": 160 },
    { "name": "Acer Predator Gaming Monitor", "price": 300, "units": 70 }
]
