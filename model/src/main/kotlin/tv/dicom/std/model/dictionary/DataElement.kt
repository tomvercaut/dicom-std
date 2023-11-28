package tv.dicom.std.model.dictionary

data class DataElement(
    var tag: RangedTag,
    var name: String,
    var keyword: String,
    var vrs: List<VR>,
    var vm: String,
    var description: String
) {

}
