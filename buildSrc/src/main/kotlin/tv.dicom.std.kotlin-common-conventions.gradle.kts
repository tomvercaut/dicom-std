plugins {
    // Apply the org.jetbrains.kotlin.jvm Plugin to add support for Kotlin.
    id("org.jetbrains.kotlin.jvm")
}

repositories {
    // Use Maven Central for resolving dependencies.
    mavenCentral()

    maven {
        url = uri("https://repository.apache.org/snapshots")
    }
}

dependencies {

    implementation("org.apache.logging.log4j:log4j-api-kotlin:1.3.0-SNAPSHOT")

    // log4j-core is a runtime dependency
    // The version of log4j-core was increased to 2.19.0 because it's binary compatible with previous releases.
    // More info can be found on:
    // - https://logging.apache.org/log4j/2.x/index.html
    // - https://logging.apache.org/log4j/kotlin/artifacts.html
    testRuntimeOnly("org.apache.logging.log4j:log4j-core:2.19.0")

    // Use the kotlin.test library which under the hood uses JUnit to run tests.
    testImplementation(kotlin("test"))
}

tasks.named<Test>("test") {
    // Use JUnit Platform for unit tests.
    useJUnitPlatform()
}
