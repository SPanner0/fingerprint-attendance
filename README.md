# Fingerprint Attendance

Take your students' attendance using a real-life fingerprint sensor.

Remove the hassle of manually taking your class's attendance by using a fingerprint sensor.
Note: This project is meant to be used with an [Arduino Uno](https://store-usa.arduino.cc/products/arduino-uno-rev3?selectedStore=us) and the [fingerprint sensor by Adafruit](https://www.adafruit.com/product/751)

## Getting Started
### 1. Set up the board and the fingerprint sensor
Hook up the fingerprint sensor to pins 2 and 3 on the Arduino board like below:
![arduino-fingerprint-setup](https://i.imgur.com/SjKXjyb.png)

### 2. Upload the fingerprint.ino file to the board
Connect the board to your computer using the USB cable (or serial, if you're old).
Use the [Official Arduino IDE](https://www.arduino.cc/en/software/) to flash the fingerprint.ino file to the board.

### 3. Run the code
Run the code using ``cargo run --release``.

You'll know it's successful if you see ``Listening on 127.0.0.1:3000``.

### 4. Enter 127.0.0.1:3000 into your browser
Enter ``127.0.0.1:3000`` into your browser's URL bar to access the attendance system.

Enjoy!

## Note: Support for other boards
As stated, this project was designed to be used with the Arduino Uno board. Other boards might work no guarantees are made; use at your own risk.
Other fingerprint sensors will NOT work, you must use the one provided by Adafruit.
