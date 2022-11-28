plugins {
    // Apply the common convention plugin for shared build configuration between library and application projects.
    id("tv.dicom.std.kotlin-common-conventions")

    // Apply the application plugin to add support for building a CLI application in Java.
    application
}
