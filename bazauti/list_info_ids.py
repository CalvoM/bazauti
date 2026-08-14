
from enum import StrEnum


class ListInfoId(StrEnum):
    """LIST INFO sub-chunk IDs.
    Ref: https://exiftool.org/TagNames/RIFF.html#Info
    """
    UNKNOWN = "Unknown"
    IARL = "Archival Location"
    IART = "Artist"
    ICMS = "Commissioned"
    ICMT = "Comments"
    ICOP = "Copyright"
    ICRD = "Creation date"
    ICRP = "Cropped"
    IDIM = "Dimensions"
    IDPI = "Dots Per Inch"
    IENG = "Engineer"
    IGNR = "Genre"
    IKEY = "Keywords"
    ILGT = "Lightness"
    IMED = "Medium"
    INAM = "Name"
    IPLT = "Palette Setting"
    IPRD = "Product"
    ISBJ = "Subject"
    ISFT = "Software"
    ISHP = "Sharpness"
    ISRC = "Source"
    ISRF = "Source Form"
    ITCH = "Technician"


def parse_list_info_id(sub_chunk_id: str) -> ListInfoId:
    match sub_chunk_id:
        case "IARL":
            return ListInfoId.IARL
        case "IART":
            return ListInfoId.IART
        case "ICMS":
            return ListInfoId.ICMS
        case "ICMT":
            return ListInfoId.ICMT
        case "ICOP":
            return ListInfoId.ICOP
        case "ICRD":
            return ListInfoId.ICRD
        case "ICRP":
            return ListInfoId.ICRP
        case "IDIM":
            return ListInfoId.IDIM
        case "IDPI":
            return ListInfoId.IDPI
        case "IENG":
            return ListInfoId.IENG
        case "IGNR":
            return ListInfoId.IGNR
        case "IKEY":
            return ListInfoId.IKEY
        case "ILGT":
            return ListInfoId.ILGT
        case "IMED":
            return ListInfoId.IMED
        case "INAM":
            return ListInfoId.INAM
        case "IPLT":
            return ListInfoId.IPLT
        case "IPRD":
            return ListInfoId.IPRD
        case "ISBJ":
            return ListInfoId.ISBJ
        case "ISFT":
            return ListInfoId.ISFT
        case "ISHP":
            return ListInfoId.ISHP
        case "ISRC":
            return ListInfoId.ISRC
        case "ISRF":
            return ListInfoId.ISRF
        case "ITCH":
            return ListInfoId.ITCH
        case _:
            return ListInfoId.UNKNOWN
