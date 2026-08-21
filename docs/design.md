# Quirpy — design notes

*This is the original design brief for Quirpy, kept here for context. It describes the intent behind
the project, not necessarily what is built today — the `plan/` folder that tracks actual
implementation work is not published. Read this for where Quirpy is going and why, and the
[README](../README.md) for what currently works.*

This program is a Rust learning curve project to learn how to build QR codes 
and how to build it by myself instead of relying on various libraries and cargo's

# Design structure
This should be an apple/windows supported application that has a simple GUI

A cursory google shows us we can select between the following QR types
- Static QR Codes: 
  Fixed data encoded directly into the pattern; they cannot be edited or tracked after creation.
- Dynamic QR Codes: Editable and trackable codes that use a short redirect URL so you can
  change the destination and view scan analytics.
- Micro QR Code: A smaller format with only one corner position marker, ideal for tight spaces like small parts or medicine packaging.
- Rectangular Micro QR (rMQR): A narrow, strip-like 2D code designed for restricted space
  constraints.
- FrameQR: Features a customizable canvas or frame area in the center for logos, text, on
  promotional images.
- SQRC (Secure QR Code): Contains restricted-reading data areas to hide private or
  confidential information requiring special authorization.
- Model 1 & Model 2: Model 1 is the original vintage prototype, while Model 2 is the modern
  global standard with alignment patterns for distorted or angled scans.

As for data types there's the choice of the following:
- URL / Website: Opens a specific web page or online destination.
- vCard / Contact: Automatically populates and saves contact details, phone numbers, and
  emails to a phone.
- Wi-Fi: Connects a device instantly to a wireless network without manual password entry
- Text / Alphanumeric: Displays a simple plain-text message or hidden code upon scanning
- Event / Calendar: Adds an event date, time, and reminder directly to a calendar app
- Email / SMS / WhatsApp: Launches a messaging or mail app pre-loaded with a recipient
  address and preset text.
- Payment: Directs the user to a secure mobile checkout or digital wallet transaction.

This means the application should allow to select a type and data-type 
I'm sure some form of MFA should be capable of creation as well.

So with that we should consider the various user-paths
For example:
User selects 'Static QR' and uses a simple 'URL' to post as data-type
How do we then move from that to actually printing out a QR code
After generation, we should allow for adjusting the color of the pixels, and whether we want a logo in this qr-code (if the size allows for it) and finally save it as an SVG, PNG, with or without background, and maybe (in the future) send to a vcard printer company. 

### What do we store in our system, or what do we store on the user's system
Personally I think the best choice is to not store anything on our servers, and only on the user's machine, with their complete control where.
We should store logs, we should store crash-reports, and ofcourse the QR exports. If the user imports an image to inject into the QR code, it has already been stored on their pc, and so we should not need to make a copy elsewhere.

### Future ideas
API to vcard printer company
pathing for the QR data, shape and form of the various pixels

### What to do first
figure out how to build a QR code manually without the need for an external library
Since most of this is a learning opportunity

## Front-end
The front-end of the app should be something relatively modern-looking. Think any apple product, or windows product with how they look. Give it a light-and-dark-mode option
The left-side of the screen should be for dropdowns and various data entries (url, wifi SSID and pw, or other data types)
wile the right-side should have a preview window previewing the created QR code
Underneath the preview it should have an export or 'save' button
the export simply saves the qr to an svg or png to the user's choice of location
the save button saves the state of the current form and type of QR code to a save file so that the user can later continue their work.
The menu bar ad the top should have a simple few menus:
_f_ile menu with a 'new' option to create a new project, an open option to open a saved file, an open recent optiont hat opens one of up to 5 recent saved files, and an exit (which closes the program without saving anything)
_e_dit with an option to undo/redo, import image which allows the user to import a small image to play as the center of the QR code, and lastly a 'preferences' option that allows the user to set some basic preferences. Like dark-mode, light-mode, default save location, etc
_h_elp which has a few options like "about" showing basic information about the program, and a 'version' button showing the current version. Lastly an 'update' button that checks the github repository for a newly released version that is higher than the current version

## back-end
the backend should build an object based on the form's inputs, from that the preview should always try to generate a QR code.
The shapes and types should regenerate the QR code, and show the result in the preview box.