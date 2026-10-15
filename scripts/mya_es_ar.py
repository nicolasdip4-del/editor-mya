#!/usr/bin/env python3
"""Editor MyA: completa la traducción al español (Argentina) de Concat.

Aplica sobre src/crates/concat/locales/es.json:
  1. Traducciones de las claves que seguían en inglés.
  2. Voseo en las frases que le hablan a la persona ("Descargá", "Elegí").
  3. Términos de uso en Argentina (video, agregar, computadora).

Correr desde la raíz del repo: python3 scripts/mya_es_ar.py
"""

import json
import re
from pathlib import Path

LOCALE = Path(__file__).resolve().parent.parent / "src/crates/concat/locales/es.json"

TEXT = {
    # ---------------------------------------------------------------- faltantes
    "captions.addCaptions": "Agregar subtítulos",
    "captions.clipNoLongerTimeline": "El clip ya no está en la línea de tiempo",
    "captions.nothingToCaptionYet": "Todavía no hay nada para subtitular",
    "captions.oneTitlePerSentence": "Un título por oración, en pantalla el tiempo necesario para leerlo, uno detrás de otro en la primera pista con lugar.",
    "captions.typeWordsPutScreen": "Escribí el texto que va en pantalla…",
    "clipInspector.audioTrack": "Pista de audio",
    "clipInspector.exposureStopsRestSigned": "La exposición va en pasos; el resto es un porcentaje con signo sobre la imagen original, y cero en todo deja la imagen como se filmó. Hacé doble clic en una etiqueta para restablecerla; el rombo pone un fotograma clave en el cabezal.",
    "clipInspector.source": "Origen",
    "effects.bleachBypass.name": "Blanqueo",
    "effects.bronze.description": "Sepia que conserva el contraste y las luces, para que parezca película vieja y no un baño marrón.",
    "effects.bronze.name": "Bronce",
    "effects.categories.cinematic": "Cine",
    "effects.categories.everyday": "Cotidiano",
    "effects.categories.film": "Película",
    "effects.categories.portrait": "Retrato",
    "effects.categories.stylised": "Estilizado",
    "effects.clean.description": "Casi imperceptible: neutro, una curva suave, un toque más de color y los rojos calmados para que la cara se vea pareja. Ideal para alguien hablando a cámara.",
    "effects.cloudlessSky.description": "Cielos más profundos y azules, vegetación más fresca y calma, el resto de la imagen y las caras sin tocar. Para videos de viaje.",
    "effects.cloudlessSky.name": "Cielo despejado",
    "effects.coolBlue.description": "Azul en sombras y medios tonos, luces y piel neutras, más contraste. Ciudades al atardecer.",
    "effects.coolBlue.name": "Azul frío",
    "effects.daydream.description": "Cine suave: negros levantados, la luz se expande cálida, sombras apenas frías, la piel intacta. El look de una historia contada junto a la ventana.",
    "effects.daydream.name": "Ensueño",
    "effects.digicam.description": "La cámara compacta de los 2000: flash duro, blancos quemados, más color, un tono cian frío, sobreenfocada y con el ruido del sensor.",
    "effects.digicam.name": "Cámara digital",
    "effects.fair.description": "Caras más luminosas y parejas: piel levantada, rojos calmados, balance neutro y el resto del cuadro sin cambios.",
    "effects.fair.name": "Luminosa",
    "effects.fallingDust.description": "Un desvanecido cálido con polvo que cae y un destello de luz en una esquina. En CPU se ve el desvanecido y un poco de ruido.",
    "effects.fallingDust.name": "Polvo",
    "effects.film.description": "Curva de película con pie y hombro reales, halo cálido alrededor de las luces y grano que se mueve. Copia de cine, no un filtro.",
    "effects.film.name": "Película",
    "effects.fresh.description": "Video de día que resalta: medios tonos levantados, más color donde está apagado, un toque menta en los blancos y la piel sin tocar.",
    "effects.fresh.name": "Fresco",
    "effects.goldenHour.description": "Luz de la tarde: balance cálido que mantiene el brillo, naranja en las luces, un susurro de magenta en las sombras y negros apenas levantados.",
    "effects.goldenHour.name": "Hora dorada",
    "effects.ivory.description": "La gemela cálida de Leche: el mismo mate suave con crema en los blancos y algo de dorado en el medio. Interiores y luz de mañana.",
    "effects.ivory.name": "Marfil",
    "effects.labels.age": "Antigüedad",
    "effects.labels.bleed": "Corrimiento",
    "effects.labels.blush": "Rubor",
    "effects.labels.brighten": "Aclarar",
    "effects.labels.chill": "Frescura",
    "effects.labels.clarity": "Claridad",
    "effects.labels.colourFilter": "Filtro de color",
    "effects.labels.dread": "Tensión",
    "effects.labels.dust": "Polvo",
    "effects.labels.even": "Emparejar",
    "effects.labels.grain": "Grano",
    "effects.labels.greens": "Verdes",
    "effects.labels.haze": "Bruma",
    "effects.labels.ink": "Tinta",
    "effects.labels.lift": "Levantar",
    "effects.labels.lightLeak": "Destello de luz",
    "effects.labels.lights": "Luces",
    "effects.labels.lines": "Líneas",
    "effects.labels.matte": "Mate",
    "effects.labels.pop": "Realce",
    "effects.labels.protectSkin": "Proteger la piel",
    "effects.labels.punch": "Intensidad",
    "effects.labels.sky": "Cielo",
    "effects.labels.soften": "Suavizar",
    "effects.labels.warmth": "Calidez",
    "effects.labels.water": "Agua",
    "effects.lagoon.description": "Agua y cielo en aguamarina y más luminosos, arena y piel cálidas y limpias. El look de playa que todos buscan primero.",
    "effects.lagoon.name": "Laguna",
    "effects.marquee.description": "Ciudad de noche: luces y carteles vivos y brillantes, negros profundos y fríos, caras limpias.",
    "effects.marquee.name": "Marquesina",
    "effects.matcha.description": "Verde suave en medios tonos y blancos, color apagado, un mate leve. El look de moda, lo bastante suave para una cara.",
    "effects.midnight.description": "Negros profundos, un tono frío en todo salvo las lámparas, poco color y algo de grano. Para videos de noche y de escritorio.",
    "effects.midnight.name": "Medianoche",
    "effects.milk.description": "Luminoso y mate: blancos suaves y fríos, negros bien levantados, color tranquilo. El look limpio y minimalista de un cuarto blanco.",
    "effects.milk.name": "Leche",
    "effects.mono.description": "Blanco y negro con filtro de color: el rojo oscurece el cielo y aclara la piel, el verde hace lo contrario. Con un poco de grano si querés.",
    "effects.moody.description": "Negros aplastados por un pie de película, color apagado, bronce en el medio, frío en las sombras y las esquinas que se cierran.",
    "effects.moody.name": "Melancólico",
    "effects.neon.description": "Magenta en las sombras, cian en las luces, contraste duro, más color y las luces que brillan. Calles de noche y carteles.",
    "effects.night.description": "Poca luz abierta: sombras levantadas con el ruido suavizado antes de subir, blancos contenidos y el azul de la noche intacto.",
    "effects.night.name": "Noche",
    "effects.peach.description": "Piel entibiada hacia durazno y suavizada, sombras levantadas para suavizar líneas, poco contraste. Retratos y gente hablando a cámara; pensado para usarse a media intensidad.",
    "effects.peach.name": "Durazno",
    "effects.pinkFuzz.description": "Poco contraste, rosa y lavanda en la luz y un brillo suave sobre todo. El soñador.",
    "effects.pinkFuzz.name": "Rosa suave",
    "effects.polaroid.description": "La foto instantánea: mate crema, luces cálidas, cian que se mete en las sombras y algo de suavidad en las esquinas.",
    "effects.rain.description": "Frío, brumoso y apagado, con los reflejos mojados levantados y brillantes. Caminatas por la calle al final del día.",
    "effects.rain.name": "Lluvia",
    "effects.retroCartoon.description": "Bandas planas de color con bordes entintados, como una historieta impresa.",
    "effects.retroCartoon.name": "Historieta retro",
    "effects.sage.description": "Apagado y verde azulado en medios tonos, una curva de película suave y calidez solo donde hay una cara. El color tranquilo del cine independiente.",
    "effects.sage.name": "Salvia",
    "effects.softLight.description": "Un mate: negros levantados, blancos bajados, poco contraste y un brillo suave y crema en la luz. Liviano y favorecedor.",
    "effects.softLight.name": "Luz suave",
    "effects.spooky.description": "Color apagado, un tono verde cian, un pie duro y una viñeta marcada. Para Halloween.",
    "effects.spooky.name": "Tenebroso",
    "effects.tealOrange.description": "Verde azulado en las sombras, naranja en las luces, la piel como piel y una curva de película encima. El color de las superproducciones de Hollywood, el look más pedido.",
    "effects.tealOrange.name": "Turquesa y naranja",
    "effects.umber.description": "El look de café: marrón cálido en el medio, verdes apagados a oliva, contraste suave. Acogedor, y favorece la madera y la piel.",
    "effects.umber.name": "Sombra tostada",
    "effects.vhs.description": "Color que se corre de lado, líneas de barrido, suavidad, ruido de cinta y demasiada saturación, con el leve arrastre vertical de un cabezal gastado.",
    "effects.vintage.description": "Una copia de los setenta: negros y blancos mate, color desvanecido, verdes que tiran a amarillo y una viñeta suave.",
    "effects.vivid.description": "Color que canta y contraste que pega, con un poco de borde: el look de una cámara mejor. La piel queda fuera del refuerzo.",
    "effects.vivid.name": "Vívido",
    "export.advanced": "Avanzado",
    "export.bitrate": "Tasa de bits",
    "export.hardware": "por hardware",
    "export.showAdvancedOptions": "Mostrar opciones avanzadas",
    "export.targetBitrate": "Tasa de bits objetivo",
    "mediaPane.import": "Importar",
    "mediaPane.importALut": "Importar una LUT (.cube)",
    "mediaPane.sortAdded": "Ordenar: agregados",
    "mediaPane.sortName": "Ordenar: nombre",
    "mediaPane.sortType": "Ordenar: tipo",
    "relink.selectFolderContainingMedia": "Elegí la carpeta con los archivos",
    "settings.playhead": "Cabezal",
    "settings.offClickRulerPast": "Desactivado, un clic en la regla después del último clip lleva el cabezal ahí, y un clip puesto en el cabezal va con él. Activado, el cabezal nunca sale del contenido.",
    "settings.scale": "Escala",
    "settings.scaleHint": "Dibuja la interfaz más grande que la escala de la pantalla. Sirve si se ve chica en un monitor grande.",
    "settings.scaleSystem": "Sistema",
    "settings.sky": "Cielo",
    "start.openProject": "Abrir proyecto",
    "studio.channels": "{0} canales",
    "studio.couldNotImport": "No se pudo importar {0}: {1}",
    "studio.importLut": "Importar LUT",
    "studio.importedLooksFindThem": "Se importaron {0} looks; están en Importados",
    "studio.mergeCancelProgress": "Cancelar unión ({0} %)",
    "studio.mergeCancelled": "Unión cancelada. Los clips originales no cambiaron.",
    "studio.mergeCouldNotReplace": "No se pudieron reemplazar los clips seleccionados.",
    "studio.mergeExportBusy": "Esperá a que termine la exportación antes de unir clips.",
    "studio.mergeFailed": "No se pudieron unir los clips: {0}",
    "studio.mergeFinished": "Clips unidos. Deshacer recupera los originales.",
    "studio.mergeSelected": "Unir clips seleccionados",
    "studio.mergeStarted": "Uniendo los clips seleccionados. Tocá Unir de nuevo para cancelar.",
    "studio.mergeTrackLocked": "Desbloqueá la pista antes de unir clips.",
    "studio.mergedClips": "Clips unidos",
    "studio.openProject": "Abrir proyecto…",
    "studio.showingLookOverPicture": "Mostrando el look sobre la imagen. Hacé doble clic en la tarjeta, o en su +, para ponerlo en la línea de tiempo",
    "studio.stereo": "estéreo",
    "studio.track": "Pista {0}",
    "textPanel.applyToAll": "Aplicar a todos",
    "textPanel.applyToAllHint": "Aplica este estilo y posición a todos los títulos de esta pista.",
    "textPanel.regular": "Normal",
    "tray.panTool": "Herramienta mano (H)",
    # ---------------------------------------------------------------- voseo
    "captions.downloadTranscriberModelFirst": "Primero descargá un modelo de transcripción en Ajustes › Transcriptor.",
    "clipInspector.nothingFoundKeepPicture": "No se encontró nada que conservar, así que la imagen se muestra tal cual. Probá con otro sujeto o con la eliminación manual.",
    "clipInspector.nothingStarredYetHover": "Todavía no hay favoritos: pasá el cursor por una tarjeta y tocá su estrella.",
    "configPane.noEffectClipYet": "Este clip todavía no tiene efectos. Elegí uno en la página Efectos de la biblioteca.",
    "keyframesPane.selectClipEditKeyframes": "Seleccioná un clip para editar sus fotogramas clave",
    "lib.pickEffectFilterLibrary": "Elegí un efecto o filtro de la biblioteca",
    "mediaPane.nothingRenderedYetRight": "Todavía no se renderizó nada. Hacé clic derecho en un clip de la línea de tiempo y renderizá su sonido para que aparezca acá.",
    "meters.clipped": "La mezcla saturó: una muestra llegó a escala completa. Hacé clic para borrar.",
    "settings.lineAcrossLanesMarks": "La línea que cruza las pistas y marca el tiempo actual. Elegí cualquier color, o el primer punto para volver al color del tema.",
    "settings.modelsFetchedFirstTime": "Los modelos se descargan la primera vez que una función los necesita. Primero se busca la descarga acá; las demás fuentes se prueban después.",
    "settings.packageOpened": "Se descargó Concat {0} y está abierto en el instalador de paquetes. Instalalo ahí y volvé a abrir el editor.",
    "settings.servesConcatApiWhile": "Sirve la API de Concat mientras la ventana está abierta, para que un script u otro programa cree, edite y exporte sus propios proyectos. Todo lo que se conecte puede leer y escribir archivos igual que vos.",
    "speech.chatterboxHasNoVoice": "Chatterbox no tiene voz propia. Activá Usar voz de muestra más abajo, o descargá Pocket TTS en Ajustes › Voz para usar sus dos grabaciones.",
    "speech.downloadVoiceModelFirst": "Primero descargá un modelo de voz en Ajustes › Voz.",
    "speech.importClipVoiceFirst": "Primero importá un clip con una voz.",
    "speech.pickARecording": "Elegí una grabación",
    "speech.pickSampleVoice": "Elegí una muestra con una voz",
    "speech.typeNarrationReadAloud": "Escribí la narración que se va a leer en voz alta…",
    "start.chooseAFolder": "Elegí una carpeta",
    "start.chooseWhereProjectFolder": "Elegí dónde va la carpeta del proyecto",
    "start.everythingHaveOpenedMachine": "Todo lo que abriste en esta computadora.",
    "studio.clipsLostAnimations": "{0} se movían con animaciones predefinidas que esta versión ya no tiene; los fotogramas clave hacen lo mismo. Guardá para conservar el cambio.",
    "studio.clipsNowForwards": "{0} se reproducían al revés en una versión anterior y ahora van hacia adelante. Seleccioná uno y tocá Invertir en la bandeja para invertirlo de nuevo; guardá para conservar el cambio.",
    "studio.selectClipTimelineFirst": "Primero seleccioná un clip en la línea de tiempo",
    "studio.selectTwoAdjacentClips": "Seleccioná dos clips que se toquen en la misma pista y después elegí una transición",
    "studio.selectVideoImageClip": "Primero seleccioná un clip de video o imagen en la línea de tiempo",
    "studio.selectVideoImageText": "Primero seleccioná un clip de video, imagen o texto en la línea de tiempo",
    "studio.timelineNowHdr": "{0} ahora es HDR (HLG), por su primer clip HDR. Podés volver atrás en Modificar.",
    "studio.upgradedEffects": "Algunos efectos de este proyecto se abren como los que los reemplazan: {0}. Guardá para conservar el cambio.",
    "textPanel.fontNotFoundTip": "Esta fuente no está instalada en esta computadora, así que el título usa una fuente de reemplazo. Instalala o elegí otra.",
    "textPanel.softShadowUnderWords": "Una sombra suave bajo las palabras, para títulos sobre imágenes que no controlás.",
    "timeline.deleteTrackClips": "Se van a eliminar esta pista y los {0} clips que tiene. Podés deshacerlo.",
    "timeline.deleteTrackEmpty": "Se va a eliminar esta pista vacía. Podés deshacerlo.",
    "timeline.deleteTrackOneClip": "Se van a eliminar esta pista y el clip que tiene. Podés deshacerlo.",
    "tray.playerFollowsResizePlayhead": "El reproductor sigue el recorte: el cabezal acompaña el borde que recortás",
    "clipInspector.chainRunsTopBottom": "La cadena va de arriba abajo: reordenala con las flechas. El ojo saltea un paso sin perder los valores que ajustaste.",
    "effects.curves.description": "Los niveles de la imagen siguen curvas que dibujás: luma solo para el brillo, después rojo, verde y azul.",
    "phone.noMediaYet": "Todavía no hay nada en la bandeja. Importá un video, un sonido o una imagen para empezar.",
    "relink.followingMediaFilesCould": "No se encontraron estos archivos. Podés volver a vincularlos a otra carpeta o cerrar este cuadro y seguir con marcadores sin conexión.",
}

# Reemplazos de vocabulario en todo el archivo (España → Argentina).
WORDS = [
    (r"\bvídeos\b", "videos"),
    (r"\bVídeos\b", "Videos"),
    (r"\bvídeo\b", "video"),
    (r"\bVídeo\b", "Video"),
    (r"\bAñadir\b", "Agregar"),
    (r"\bañadir\b", "agregar"),
    (r"\bañadidos\b", "agregados"),
    (r"\bañadido\b", "agregado"),
    (r"\bañadidas\b", "agregadas"),
    (r"\bañadida\b", "agregada"),
    (r"\bAñade\b", "Agrega"),
    (r"\bañade\b", "agrega"),
    (r"\bordenador\b", "computadora"),
    (r"\bAún no\b", "Todavía no"),
    (r"\baún no\b", "todavía no"),
]


def main() -> None:
    raw = LOCALE.read_bytes().decode("utf-8")
    newline = "\r\n" if "\r\n" in raw else "\n"
    data = json.loads(raw)

    unknown = [key for key in TEXT if key not in data]
    if unknown:
        raise SystemExit(f"claves inexistentes: {unknown}")

    data.update(TEXT)
    for key, value in data.items():
        if not isinstance(value, str):
            continue
        for pattern, replacement in WORDS:
            value = re.sub(pattern, replacement, value)
        data[key] = value

    text = json.dumps(data, ensure_ascii=False, indent=2) + "\n"
    LOCALE.write_bytes(text.replace("\n", newline).encode("utf-8"))
    print(f"{len(TEXT)} frases actualizadas en {LOCALE.name}")


if __name__ == "__main__":
    main()
