/***********************************************************************************************************************
ATTENTION: This file is meant to be flashed to the Arduino Uno (other boards might work but there is no guarantee).
This project is meant to be used with the fingerprint sensor module from Adafruit (https://www.adafruit.com/product/751).
This file will NOT be compiled by Cargo; rather, it is meant to be compiled by the Arduino IDE or similar Arduino uploaders.
***********************************************************************************************************************/

#include <Adafruit_Fingerprint.h>
#include <stdint.h>

SoftwareSerial fingerprint_serial(2, 3);
Adafruit_Fingerprint fingerprint_sensor = Adafruit_Fingerprint(&fingerprint_serial);

const uint32_t PORT_BAUD_RATE = 9600;
const uint32_t FINGER_BAUD_RATE = 57600;

enum Command {
  Ready = 0,
  Enroll = 1,
  Match = 2,
  Clear = 3,
};

enum CommandResponse {
    UnknownError = 1,
    PacketError = 2,
    ImageFail = 3,
    ImageMess = 4,
    FeatureFail = 5,
    InvalidImage = 6,
    EnrollMismatch = 7,
    BadLocation = 8,
    FlashError = 9,
    FingerprintNotFound = 10,
    UnknownCommand = 11,
};

enum CommandStatus {
    Success = 0b11111111,
    Failure = 0b11111110,
};

// Function prototypes
void setup();
void loop();
void run_command(uint8_t[2], uint8_t(&)[2]);
void check_ready(uint8_t (&)[2]);
void enroll_fingerprint(uint8_t, uint8_t(&)[2]);
void match_fingerprint(uint8_t(&)[2]);
void clear_fingerprints(uint8_t(&)[2]);
bool process_get_image(uint8_t(&)[2]);
bool process_template_convert(uint8_t(&)[2], uint8_t);
void serial_flush();

bool ready = false; // Whether the fingerprint sensor is ready to be used
void setup()
{
    Serial.begin(PORT_BAUD_RATE);
    delay(100);
    
    fingerprint_sensor.begin(FINGER_BAUD_RATE);

    ready = fingerprint_sensor.verifyPassword();

    pinMode(LED_BUILTIN, OUTPUT); // For debugging purposes
}

void loop()
{
    // Commands will always come in an array of size 2
    // The first byte is the command, the second any data the command needs
    uint8_t command[2];
    fingerprint_sensor.LEDcontrol(FINGERPRINT_LED_ON, 0, FINGERPRINT_LED_BLUE);

    if (Serial.available() > 0)
    {
        Serial.readBytes(command, 2);
        uint8_t command_response[2];      // Used to send a response back to the computer
        command_response[0] = CommandStatus::Failure; // Set the response code to failure by default

        run_command(command, command_response);
        Serial.write(command_response, 2);

        serial_flush();
        delay(3000);
    }
}

/// @brief Run a command based on the command byte
/// @param command An array of size 2 containing the command and any data the command needs
/// @param command_response An array to store the command response and write back to the computer
void run_command(uint8_t command[2], uint8_t (&command_response)[2])
{
    switch (command[0])
    {
    case Command::Ready: // Start up acknowledgement
        check_ready(command_response);
        return;
    case Command::Enroll:
        enroll_fingerprint(command[1], command_response);
        return;
    case Command::Match:
        match_fingerprint(command_response);
        return;
    case Command::Clear:
        clear_fingerprints(command_response);
        return;
    default:
        // Unknown command
        command_response[0] = CommandStatus::Failure;
        command_response[1] = CommandResponse::UnknownError;
        return;
    }
}

/// @brief Check if the fingerprint sensor is ready to be used
/// @param command_response An array to store the command response and write back to the computer
void check_ready(uint8_t (&command_response)[2]) {
    if (ready) {
        command_response[0] = CommandStatus::Success;
  } else {
        command_response[0] = CommandStatus::Failure;
  }
}

/// @brief Enroll a fingerprint with a specified ID
/// @param fingerprint_id The ID to store the fingerprint under
/// @param command_response An array to store the command response and write back to the computer
void enroll_fingerprint(uint8_t fingerprint_id, uint8_t (&command_response)[2])
{
    // Take the first image
    if (!process_get_image(command_response))
    {
        return;
    }

    // Convert the first image to a template
    if (!process_template_convert(command_response, 1))
    {
        return;
    }

    fingerprint_sensor.LEDcontrol(FINGERPRINT_LED_OFF, 0, FINGERPRINT_LED_BLUE);
    // Wait for the user to remove the finger
    delay(2000);
    int status = 0;
    while (status != FINGERPRINT_NOFINGER)
    {
        status = fingerprint_sensor.getImage();
    }

    fingerprint_sensor.LEDcontrol(FINGERPRINT_LED_GRADUAL_ON, 0, FINGERPRINT_LED_BLUE);
    // Take the second image
    if (!process_get_image(command_response))
    {
        return;
    }

    // Save the second template
    if (!process_template_convert(command_response, 2))
    {
        return;
    }

    // Create the model using the two saved templates
    status = fingerprint_sensor.createModel();
    switch (status)
    {
    case FINGERPRINT_OK:
        break;
    case FINGERPRINT_PACKETRECIEVEERR:
        fingerprint_sensor.LEDcontrol(FINGERPRINT_LED_ON, 0, FINGERPRINT_LED_RED);
        command_response[1] = CommandResponse::PacketError;
        return;
    case FINGERPRINT_ENROLLMISMATCH:
        fingerprint_sensor.LEDcontrol(FINGERPRINT_LED_ON, 0, FINGERPRINT_LED_RED);
        command_response[1] = CommandResponse::EnrollMismatch;
        return;
    default:
        fingerprint_sensor.LEDcontrol(FINGERPRINT_LED_ON, 0, FINGERPRINT_LED_RED);
        command_response[1] = CommandResponse::UnknownError;
        return;
    }

    // Store the model with the ID given by the computer
    status = fingerprint_sensor.storeModel(fingerprint_id);
    switch (status)
    {
    case FINGERPRINT_OK:
        break;
    case FINGERPRINT_PACKETRECIEVEERR:
        fingerprint_sensor.LEDcontrol(FINGERPRINT_LED_ON, 0, FINGERPRINT_LED_RED);
        command_response[1] = CommandResponse::PacketError;
        return;
    case FINGERPRINT_BADLOCATION:
        fingerprint_sensor.LEDcontrol(FINGERPRINT_LED_ON, 0, FINGERPRINT_LED_RED);
        command_response[1] = CommandResponse::BadLocation;
        return;
    case FINGERPRINT_FLASHERR:
        fingerprint_sensor.LEDcontrol(FINGERPRINT_LED_ON, 0, FINGERPRINT_LED_RED);
        command_response[1] = CommandResponse::FlashError;
        return;
    default:
        command_response[1] = CommandResponse::UnknownError;
        return;
    }

    fingerprint_sensor.LEDcontrol(FINGERPRINT_LED_ON, 500, FINGERPRINT_LED_PURPLE);
    // If it reaches here, the operation succeeded
    command_response[0] = CommandStatus::Success;
    command_response[1] = fingerprint_id;
    return;
}

/// @brief Scan a fingerprint and try to match it with one in the database
/// @param command_response An array to store the command response and write back to the computer
void match_fingerprint(uint8_t (&command_response)[2])
{
    if (!process_get_image(command_response))
    {
        return;
    }

    if (!process_template_convert(command_response, 1))
    {
        return;
    }

    // Search for the fingerprint in the template saved in slot 1
    int status = fingerprint_sensor.fingerSearch();
    switch (status)
    {
    case FINGERPRINT_OK:
        fingerprint_sensor.LEDcontrol(FINGERPRINT_LED_ON, 0, FINGERPRINT_LED_BLUE);
        command_response[0] = CommandStatus::Success;
        command_response[1] = (uint8_t) fingerprint_sensor.fingerID;
        break;
    case FINGERPRINT_PACKETRECIEVEERR:
        fingerprint_sensor.LEDcontrol(FINGERPRINT_LED_ON, 0, FINGERPRINT_LED_RED);
        command_response[1] = CommandResponse::PacketError;
        break;
    case FINGERPRINT_NOTFOUND:
        fingerprint_sensor.LEDcontrol(FINGERPRINT_LED_ON, 0, FINGERPRINT_LED_RED);
        command_response[1] = CommandResponse::FingerprintNotFound;
        break;
    default:
        fingerprint_sensor.LEDcontrol(FINGERPRINT_LED_ON, 0, FINGERPRINT_LED_RED);
        command_response[1] = CommandResponse::UnknownError;
        break;
    }

    return;
}

/// @brief Clear all fingerprints from the database
/// @param command_response An array to store the command response and write back to the computer
void clear_fingerprints(uint8_t (&command_response)[2])
{
    int status = fingerprint_sensor.emptyDatabase();
    switch (status)
    {
    case FINGERPRINT_OK:
        command_response[0] = CommandStatus::Success;
        return;
    case FINGERPRINT_PACKETRECIEVEERR:
        fingerprint_sensor.LEDcontrol(FINGERPRINT_LED_ON, 0, FINGERPRINT_LED_RED);
        command_response[1] = CommandResponse::PacketError;
        return;
    case FINGERPRINT_BADLOCATION:
        fingerprint_sensor.LEDcontrol(FINGERPRINT_LED_ON, 0, FINGERPRINT_LED_RED);
        command_response[1] = CommandResponse::BadLocation;
        return;
    case FINGERPRINT_FLASHERR:
        fingerprint_sensor.LEDcontrol(FINGERPRINT_LED_ON, 0, FINGERPRINT_LED_RED);
        command_response[1] = CommandResponse::FlashError;
        return;
    default:
        fingerprint_sensor.LEDcontrol(FINGERPRINT_LED_ON, 0, FINGERPRINT_LED_RED);
        command_response[1] = CommandResponse::UnknownError;
        return;
    }
}

/// @brief Helper function to get scan image from the fingerprint sensor
/// @param command_response An array to store the command response and write back to the computer
/// @return Whether the operation succeeded
bool process_get_image(uint8_t (&command_response)[2])
{
    int status = -1;

    while (status != FINGERPRINT_OK)
    {
        status = fingerprint_sensor.getImage();
        switch (status)
        {
        case FINGERPRINT_OK:
            return true;
        case FINGERPRINT_NOFINGER:
            break;
        case FINGERPRINT_PACKETRECIEVEERR:
            fingerprint_sensor.LEDcontrol(FINGERPRINT_LED_ON, 0, FINGERPRINT_LED_RED);
            command_response[1] = CommandResponse::PacketError;
            return false;
        case FINGERPRINT_IMAGEFAIL:
            fingerprint_sensor.LEDcontrol(FINGERPRINT_LED_ON, 0, FINGERPRINT_LED_RED);
            command_response[1] = CommandResponse::ImageFail;
            return false;
        default:
            fingerprint_sensor.LEDcontrol(FINGERPRINT_LED_ON, 0, FINGERPRINT_LED_RED);
            command_response[1] = CommandResponse::UnknownError;
            return false;
        }
    }
}

/// @brief Helper function to convert a taken fingerprint image to a template
/// @param command_response An array to store the command response and write back to the computer
/// @param slot The slot to store the template in
/// @return Whether the operation succeeded
bool process_template_convert(uint8_t (&command_response)[2], uint8_t slot)
{
    int status = fingerprint_sensor.image2Tz(slot);
    switch (status)
    {
    case FINGERPRINT_OK:
        return true;
    case FINGERPRINT_IMAGEMESS:
        fingerprint_sensor.LEDcontrol(FINGERPRINT_LED_ON, 0, FINGERPRINT_LED_RED);
        command_response[1] = 4;
        return false;
    case FINGERPRINT_PACKETRECIEVEERR:
        fingerprint_sensor.LEDcontrol(FINGERPRINT_LED_ON, 0, FINGERPRINT_LED_RED);
        command_response[1] = 2;
        return false;
    case FINGERPRINT_FEATUREFAIL:
        fingerprint_sensor.LEDcontrol(FINGERPRINT_LED_ON, 0, FINGERPRINT_LED_RED);
        command_response[1] = 5;
        return false;
    case FINGERPRINT_INVALIDIMAGE:
        fingerprint_sensor.LEDcontrol(FINGERPRINT_LED_ON, 0, FINGERPRINT_LED_RED);
        command_response[1] = 6;
        return false;
    default:
        fingerprint_sensor.LEDcontrol(FINGERPRINT_LED_ON, 0, FINGERPRINT_LED_RED);
        command_response[1] = 1;
        return false;
    }
}

/// @brief Flush the serial read buffer
void serial_flush() {
  while(Serial.available() > 0) {
    char t = Serial.read();
  }
}