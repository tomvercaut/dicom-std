package tv.dicom.std.model

import org.apache.logging.log4j.kotlin.logger
import java.util.Locale
import java.util.ResourceBundle

/**
 * Helper class to manage ResourceBundles used in the project.
 */
class Resource {
    companion object {
        @JvmStatic
        private val log = logger(this::class.java.name)

        @JvmStatic
        private var _errorResourceBundle: ResourceBundle? = null

        /**
         * Get the ResourceBundle with error messages.
         */
        @JvmStatic
        val error: ResourceBundle
            get() {
                if (_errorResourceBundle == null) {
                    val local = Locale.getDefault() ?: Locale.ROOT
                    if (local != null) {
                        _errorResourceBundle = ResourceBundle.getBundle("errors", local)
                    } else {
                        log.error("Unable to get default Locale.")
                        _errorResourceBundle = ResourceBundle.getBundle("errors")
                    }
                }
                if (_errorResourceBundle == null) {
                    log.error("Error ResourceBundle is null")
                    throw NullPointerException("Internal resource bundle is null.")
                }
                return _errorResourceBundle!!
            }

        /**
         * Get a message from an error ResourceBundle.
         *
         * @param key resource property key
         * @return The error message from the ResourceBundle. An empty error message is returned if a NullPointerException is caught internally.
         */
        @JvmStatic
        fun errorMessage(key: String): String {
            return try {
                val rb = error
                rb.getString(key)
            } catch (ex: NullPointerException) {
                ""
            }
        }
    }
}