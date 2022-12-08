package tv.dicom.std.core.model

import tv.dicom.std.core.model.ciod.Ciod
import tv.dicom.std.core.model.imd.Imd

class DicomStandard {
    var ciods: Map<String, Ciod> = mapOf()
    var imds: Map<String, Imd> = mapOf()

    /**
     * Check if a Composite Information Object Definition, Information Module Definition or other (macro attribute) definition exists using the XML table id.
     *
     * @param id XML table identifier
     * @return True if a matching definition is found, false otherwise
     */
    internal fun has(id: String): Boolean {
        if (ciods.containsKey(id)) {
            return true
        }
        if (imds.containsKey(id)) {
            return true
        }
        return false
    }
}
