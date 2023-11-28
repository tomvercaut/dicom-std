package tv.dicom.std.model

import org.junit.jupiter.api.Test

import org.junit.jupiter.api.Assertions.*

class ResourceTest {

    @Test
    fun getError() {
        val expected = "ResourceTest"
        val value = Resource.error.getString("ResourceTest")
        assertEquals(expected, value)
    }

    @Test
    fun getErrorMessage() {
        val expected = "ResourceTest"
        val value = Resource.errorMessage("ResourceTest")
        assertEquals(expected, value)
    }
}