package tv.dicom.std.core.model

import tv.dicom.std.core.model.ciod.Ciod
import tv.dicom.std.core.model.imd.Imd

class DicomStandard {
    private val ciods: MutableMap<String, Ciod> = mutableMapOf()
    private val imds: MutableMap<String, Imd> = mutableMapOf()

    /**
     * Add a Composite Information Object Definition to the DicomStandard.
     *
     * @param ciod Composite Information Object Definition instance
     * @return True the instance is added. False if the instance was not added.
     */
    fun add(ciod: Ciod): Boolean {
        if (!ciods.containsKey(ciod.id)) {
            ciods[ciod.id] = ciod
            return true
        }
        return false
    }

    /**
     * Add an Information Module Definition to the DicomStandard.
     *
     * @param imd Information Module Definition instance
     * @return True the instance is added. False if the instance was not added.
     */
    fun add(imd: Imd): Boolean {
        if (!imds.containsKey(imd.id)) {
            imds[imd.id] = imd
            return true
        }
        return false
    }

    /**
     * Get a [Set] of Composite Information Object Definition IDs which match the XML table IDs in the DICOM standard part 03.
     *
     * @return A [Set] of XML table IDs linked to Composite Information Object Definition IDs
     */
    fun ciodIds(): Set<String> {
        return ciods.keys
    }

    /**
     * Get a [Set] of Information Module Definition IDs which match the XML table IDs in the DICOM standard part 03.
     *
     * @return A [Set] of XML table IDs linked to Information Module Definition IDs
     */
    fun imdIds(): Set<String> {
        return imds.keys
    }

    /**
     * Get a Composite Information Object Definition by XML table ID.
     *
     * @param id XML table ID
     * @return A Composite Information Object Definition, null is returned if the key doesn't have a matching CIOD.
     */
    fun ciod(id: String): Ciod? {
        return ciods[id]
    }

    /**
     * Get an Information Module Definition by XML table ID.
     *
     * @param id XML table ID
     * @return An Information Module Definition, null is returned if the key doesn't have a matching IMD.
     */
    fun imd(id: String): Imd? {
        return imds[id]
    }

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
