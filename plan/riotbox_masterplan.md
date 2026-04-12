# Riotbox — Masterplan
**A feral terminal audio instrument**

Version: 0.1  
Status: Strategischer und technischer Implementierungsplan  
Sprache: Deutsch

---

## Inhaltsverzeichnis

- [1. Zielbild](#1-zielbild)
- [2. Produktdefinition](#2-produktdefinition)
- [3. Leitprinzipien](#3-leitprinzipien)
- [4. Soundidentität: die drei Geräte-Persönlichkeiten](#4-soundidentität-die-drei-geräte-persönlichkeiten)
- [5. Betriebsmodi](#5-betriebsmodi)
- [6. Zielgruppen und Kern-Use-Cases](#6-zielgruppen-und-kern-use-cases)
- [7. Funktionsumfang auf Produktebene](#7-funktionsumfang-auf-produktebene)
- [8. Systemarchitektur](#8-systemarchitektur)
- [9. Technologie-Stack](#9-technologie-stack)
- [10. Audiodatenfluss](#10-audiodatenfluss)
- [11. Analyse-Pipeline](#11-analyse-pipeline)
- [12. Interne musikalische Repräsentationen](#12-interne-musikalische-repräsentationen)
- [13. Geräte-Engine-Design](#13-geräte-engine-design)
- [14. Arrangement und Kompositionslogik](#14-arrangement-und-kompositionslogik)
- [15. Scoring und Auswahl](#15-scoring-und-auswahl)
- [16. Live-Mutation und Performance-Steuerung](#16-live-mutation-und-performance-steuerung)
- [17. KI-Agent / Ghost-System](#17-ki-agent--ghost-system)
- [18. UX-Strategie](#18-ux-strategie)
- [19. TUI-Konzept](#19-tui-konzept)
- [20. Controller- und Hardware-Integration](#20-controller--und-hardware-integration)
- [21. Sessionmodell](#21-sessionmodell)
- [22. Datenmodell](#22-datenmodell-vereinfachte-skizze)
- [23. Capture und Looping als Herzstück](#23-capture-und-looping-als-herzstück)
- [24. Resample-Lab](#24-resample-lab)
- [25. FX- und Mixer-Strategie](#25-fx--und-mixer-strategie)
- [26. Export und Interoperabilität](#26-export-und-interoperabilität)
- [27. Qualitätssicherung und Profi-Härtung](#27-qualitätssicherung-und-profi-härtung)
- [28. Provider-Architektur](#28-provider-architektur)
- [29. Recht, Lizenzen, Originalität](#29-recht-lizenzen-originalität)
- [30. Repository-Struktur](#30-repository-struktur)
- [31. MVP-Definition](#31-mvp-definition)
- [32. Umsetzungsphasen](#32-umsetzungsphasen)
- [33. Detaillierter Backlog nach Modulen](#33-detaillierter-backlog-nach-modulen)
- [34. Nichtfunktionale Anforderungen](#34-nichtfunktionale-anforderungen)
- [35. Preset- und Style-System](#35-preset--und-style-system)
- [36. Preset-Makros](#36-preset-makros)
- [37. Explainability und Vertrauen](#37-explainability-und-vertrauen)
- [38. Risikoanalyse](#38-risikoanalyse)
- [39. Team- und Rollenempfehlung](#39-team--und-rollenempfehlung)
- [40. Erste Meilensteine in Wochenlogik](#40-erste-meilensteine-in-wochenlogik)
- [41. Entscheidungsregeln für die Entwicklung](#41-entscheidungsregeln-für-die-entwicklung)
- [42. Schlussbild](#42-schlussbild)
- [43. Nächste konkrete Dokumente](#43-nächste-konkrete-dokumente)
- [44. Ein-Satz-Version](#44-ein-satz-version)

---


## 1. Zielbild

**Riotbox** ist ein terminal-natives Audioinstrument für Live-Performance, Sound-Mutation und kontrollierte Re-Komposition.  
Der Benutzer lädt ein Audiofile (z. B. MP3), Riotbox analysiert dessen musikalische Struktur und übersetzt es in ein spielbares Live-Objekt. Dieses Objekt wird anschließend durch drei klar definierte Geräte-Persönlichkeiten neu aufgebaut:

- **MC-202** als monophone Sequenz- und Synth-Nervenzentrale
- **W-30** als Sampler-, Slice- und Resampling-Maschine
- **TR-909** als Drum- und Groove-Motor

Das Ergebnis ist **kein Black-Box-Audio-zu-Audio-Zaubertrick**, sondern ein **kontrollierbares Instrument**, das:

- das Eingangsmaterial analysiert,
- musikalische Repräsentationen erzeugt,
- daraus live variierbare Szenen, Loops und Phrasen baut,
- spontan mutiert werden kann,
- von einem lokalen KI-Agenten assistiert oder performt werden kann,
- reproduzierbar speicherbar und exportierbar bleibt.

---

## 2. Produktdefinition

### 2.1 Was Riotbox ist
Riotbox ist ein **professionelles Sound-Tool** mit drei Gesichtern:

1. **Live-Remix-Instrument**  
   Audio rein, Analyse, Rebuild, Performance.

2. **Sound-Design-Maschine**  
   Slices, Resampling, Phrase-Capture, Layering, Mutation.

3. **Autonomer Mitspieler**  
   Ein lokaler KI-Agent kann Vorschläge machen oder das Instrument live bedienen.

### 2.2 Was Riotbox nicht ist
Riotbox ist **nicht**:

- eine Text-prompt-basierte “mach mir einen Song”-Spielerei,
- ein undurchschaubares End-to-End-Audio-Transfer-Modell,
- ein klassisches DAW-Ersatzsystem,
- ein Tool, das nur bestehendes Material 1:1 imitiert,
- ein System, das einen Expertenabschluss verlangt, um musikalisch Spaß zu machen.

### 2.3 Kernsatz
> Riotbox verwandelt Audio in ein lebendiges Performance-Objekt und erlaubt es, dieses Objekt in Echtzeit mit Geräte-Charakter, Szenenlogik, Capture und KI-Co-Performance zu formen.

---

## 3. Leitprinzipien

### 3.1 Instrument statt Black Box
Alle wichtigen musikalischen Entscheidungen müssen über **sichtbare Zustände, Aktionen und Makros** erreichbar sein.

### 3.2 Realtime first
Die Audio-Engine muss auch dann stabil bleiben, wenn Analyse, KI oder Dateiverarbeitung versagen.

### 3.3 Progressive Tiefe
Die Bedienung muss in Schichten funktionieren:

- **Jam**: sofort spielen
- **Sculpt**: gezielt formen
- **Lab**: tief analysieren und debuggen

### 3.4 Wiederholbarkeit
Jede Session ist deterministisch speicherbar:

- globale Seeds
- lokale Seeds
- Aktionslog
- Analyse-Cache
- Szenen- und Capture-Historie

### 3.5 Musikalische Brauchbarkeit vor akademischer Vollständigkeit
Das System muss **gut klingen, schnell reagieren und Spaß machen**. Wissenschaftlich perfekte Analyse ist zweitrangig, wenn musikalisch gute Entscheidungen bereits möglich sind.

### 3.6 Originalität mit Strukturbezug
Riotbox soll Material **inspiriert von Struktur**, nicht blind **kopierend nach Oberfläche** erzeugen. Dafür gibt es klare Betriebsmodi und eine spätere Similarity-Firewall.

---

## 4. Soundidentität: die drei Geräte-Persönlichkeiten

### 4.1 MC-202-Rolle
Die MC-202-Schicht ist der **Mono-Nerv** des Systems.  
Sie übernimmt:

- Basslines
- kurze Leads
- Antwortphrasen
- Akzentuierung
- Slides/Glides
- monotone, bedrohliche Repetitionen
- harmoniestützende oder gegenläufige Figuren

**Leitbild:** wenige Noten, hohe Aussagekraft.

### Klangcharakter
- monophon
- prägnante Sequenzierung
- portamento/glide
- accent-gesteuerte Energie
- filtergetragene Spannung
- leicht nervöse, treibende Präsenz

### Bedienphilosophie
Die 202-Engine darf nie zu busy werden. Sie soll **Rückgrat und Biss** liefern, nicht den kompletten Mix überfahren.

### 4.2 W-30-Rolle
Die W-30-Schicht ist die **Sampler-Seele**.

Sie übernimmt:

- Slice-Erzeugung
- Loop-Erkennung
- Pad-Bänke
- Resampling
- Re-Pitching
- Reverse
- Bit-/Rate-Charakter
- Phrase-Capture
- Self-sampling interner Klangerzeuger

**Leitbild:** alles, was musikalisch interessant ist, muss eingefroren, neu belegbar und wiederverwendbar werden können.

### 4.3 TR-909-Rolle
Die TR-909-Schicht ist der **Drum-Motor**.

Sie übernimmt:

- Drum-Reinforcement
- Kick-/Snare-Layering
- Pattern-Rebuild
- Accent-Mapping
- Hi-Hat-Verdichtung
- Fills
- Drop-Vorbereitung
- Drum-Bus-Energie

**Leitbild:** Punch, Präzision, Groove-Spannung und kontrollierte Aggression.

---

## 5. Betriebsmodi

### 5.1 Derivative Mode
Der Eingang bleibt hörbar erhalten. Riotbox arbeitet mit echtem Material aus dem Input.

Verwendet:
- reale Loops
- reale Slices
- reale Texturen
- reale Vox-Fragmente
- echtes Timing-Material

**Einsatz:** Live-Edit, Mutation, Re-Cut, DJ-nahe Performance.

### 5.2 De Novo Mode
Der Input liefert nur Struktur, nicht zwingend direktes Audiomaterial.

Verwendet:
- BPM / Grid
- Tonart / Akkorde
- Melodiekonturen
- Energieverläufe
- Abschnittslogik
- Embeddings / Stilvektoren

Erzeugt:
- neue Drum-Patterns
- neue 202-Phrasen
- neue Hook-Samples
- neue interne Loops und Szenen

**Einsatz:** inspirierter Neuaufbau, stilistisch nahe, materiell eigenständiger.

### 5.3 Hybrid Mode
Kombiniert beide Welten:
- Originaldrums ersetzen, aber Vocals behalten
- Original-Hook slicen, aber mit neuer Bassline
- internes Resampling auf Basis extrahierter Phrasen

**Das ist voraussichtlich der wichtigste Live-Modus.**

---

## 6. Zielgruppen und Kern-Use-Cases

### 6.1 Live-Performer
- lädt Track
- startet Riotbox
- lässt ein Rebuild erzeugen
- steuert Energie, Loops, Szenen und Mutationen live
- captured gelungene Momente

### 6.2 Produzent
- extrahiert gute Loops und Phrasen
- baut daraus neue Hook-Bänke
- exportiert Stems und MIDI
- nutzt Ghost/AI als Co-Sound-Designer

### 6.3 Explorateur / Zuhörer
- lädt Material
- schaltet `ghost=perform`
- beobachtet das Instrument
- hört, wie der lokale Agent musikalische Entscheidungen trifft

### 6.4 Sound-Designer
- zerlegt eigenes Material
- nutzt W-30-Capture und Resample Lab
- erstellt charaktervolle Sample-Banken

---

## 7. Funktionsumfang auf Produktebene

Riotbox muss in seiner finalen Form mindestens Folgendes beherrschen:

- Audiofile laden
- Analyse-Cache erzeugen
- Taktgitter bestimmen
- Tonart/Akkorde schätzen
- Stems oder Teilkomponenten isolieren
- Slice- und Loop-Kandidaten finden
- interne Geräte-Engines speisen
- Live-Szenen generieren
- Loops extrahieren und einfrieren
- sofortige Live-Mutation
- Undo / Revert / Snapshot
- KI-Vorschläge und KI-Performance
- MIDI/HID-Steuerung
- Session-Speicherung
- Stereo- und Stem-Export
- deterministische Wiederherstellung
- Performance-Logging und Diagnose

---

## 8. Systemarchitektur

```text
                  +-----------------------------+
                  |        Terminal UI          |
                  |  Jam / Sculpt / Lab / Log   |
                  +-------------+---------------+
                                |
                                v
                    +-----------+-----------+
                    |      Session Core      |
                    | state / actions / undo |
                    +----+---------------+---+
                         |               |
            control bus  |               | scheduled events
                         v               v
                +--------+----+   +------+--------+
                | Composer /   |   | Realtime     |
                | Scene Brain  |   | Audio Engine |
                +--------+----+   +------+--------+
                         |               |
                         |               v
                         |         audio output
                         |
                         v
                +--------+---------------------------+
                |  Device Engines                    |
                |  MC-202 / W-30 / TR-909 / FX       |
                +--------+---------------------------+
                         ^
                         |
                +--------+---------------------------+
                | Analysis + AI Sidecar              |
                | stems / beats / chords / embeddings|
                | loop mining / local agent          |
                +------------------------------------+
```

### 8.1 Prozessgrenzen
Riotbox wird in mindestens zwei Hauptprozesse getrennt:

### Realtime Core
Verantwortlich für:
- Audio-Thread
- Scheduler
- Mixer
- Gerätelogik
- Terminal UI
- Controller Input
- Session State
- Undo/Snapshots

### Analysis + AI Sidecar
Verantwortlich für:
- Dekodierung vorbereiteter Dateien
- Stem Separation
- Beat-/Bar-Erkennung
- Chord-/Key-Erkennung
- Loop-Mining
- Embeddings
- Kandidatenscoring
- lokaler KI-Agent
- Offline/Background-Analyse

**Regel:** Der Sidecar darf sterben, die Audio-Engine nicht.

### 8.2 Architekturprinzip
- **Realtime und nicht-realtime strikt trennen**
- **Mutationen nur an sicheren Quantisierungsgrenzen committen**
- **Keine Heap-Allokationen im Audio-Callback**
- **Alle schweren Modelle, Decoder und Analysen out-of-process**

---

## 9. Technologie-Stack

### 9.1 Realtime Core
Empfohlen: **Rust**

Gründe:
- gute Kontrolle über Speicher und Realtime-Verhalten
- native Performance
- sichere Concurrency
- gute TUI- und Audio-Bibliotheken

### Kernkomponenten
- Audio I/O
- TUI
- MIDI/HID
- Scheduler
- DSP
- Session Serialization
- Export-Subsystem

### 9.2 Analysis + AI
Empfohlen: **Python-Sidecar**

Gründe:
- MIR-Ökosystem
- Modellausführung
- schnellere Iteration
- leichtere Forschung und Prototyping

### Kernkomponenten
- Audio-Vorverarbeitung
- Modellprovider
- Feature-Extraktion
- RPC-Server
- lokaler Agent
- Kandidaten- und Bewertungslogik

### 9.3 Kommunikation
Zwischen Rust-Core und Python-Sidecar:

- lokale Unix-Sockets oder TCP auf localhost
- MessagePack / Protobuf / JSON-RPC
- asynchrones Action-Protokoll
- Versionsnummern in allen Nachrichten

---

## 10. Audiodatenfluss

```text
Input Audio
 -> decode
 -> normalize
 -> optional loudness alignment
 -> stem separation
 -> beat/downbeat/bar grid
 -> key/chord estimation
 -> note/bass contour extraction
 -> section segmentation
 -> onset/transient slicing
 -> loop candidate mining
 -> embeddings / similarity vectors
 -> Source Graph
 -> Scene Graph + device feeds
 -> live playback / mutation / capture
 -> export
```

---

## 11. Analyse-Pipeline

### 11.1 Decode & Normalize
### Aufgaben
- MP3/WAV/AIFF/FLAC einlesen
- in interne Samplerate wandeln
- Kanäle normalisieren
- Loudness für Analyse stabilisieren

### Ziele
- konsistente Eingangsdaten
- keine Überraschungen in späterer Feature-Extraktion

### 11.2 Stem Separation
### Ziel
Trennung in:
- drums
- bass
- vocals
- harmonic/rest
- optional FX/noise

### Nutzen
- saubere Drum-Detektion
- bessere Tonhöhenanalyse
- gezielte Geräte-Zuweisung

### 11.3 Beat, Downbeat, Bars
### Output
- BPM-Kandidat(en)
- Beat-Frames
- Downbeats
- Bar-Grenzen
- Confidence-Werte
- alternative Grid-Hypothesen

### Wichtig
Bar-Grid ist zentrale Wahrheit für:
- Szenenwechsel
- Loop-Capture
- quantisierte Mutationen
- Ghost-Actions

### 11.4 Harmonieanalyse
### Output
- Tonart
- Modus
- Akkordfenster pro Bar/Halbbart
- Bass-Zentren
- funktionale Spannung

### Nutzung
- MC-202-Follower
- Hook-Resynthese
- De Novo Rebuild

### 11.5 Melodie- und Konturextraktion
### Output
- Lead-/Hook-Konturen
- Basskonturen
- Phrasenanker
- intervallische Muster

### Nutzung
- 202-Phrasen
- Motivantworten
- similarity-aware Mutation

### 11.6 Struktursegmentierung
### Output
- Intro / Build / Drop / Breakdown / Outro-Kandidaten
- Energieverlauf
- Wiederholungsblöcke
- Übergangsstellen

### Nutzung
- Scene Graph
- Ghost-Vorschläge
- Auto-Arrangement

### 11.7 Slice- und Loop-Mining
### Output
- transientbasierte Slices
- musikalische 1/2/4/8-Bar-Loops
- Pad-Kandidaten
- One-Shot-Kandidaten
- Qualitätsscores

### Qualitätskriterien
- rhythmische Schlüssigkeit
- geringer Artefaktgrad
- harmonische Eindeutigkeit
- Wiederverwendbarkeit
- Live-Nützlichkeit

### 11.8 Embeddings und Stilvektoren
### Output
- dichte Repräsentationen für Abschnitte
- Similarity-Mapping innerhalb des Tracks
- Charakter- und Energieverläufe
- Cluster ähnlicher Passagen

### Nutzung
- intelligente Abschnittssprünge
- Ähnlichkeitsvermeidung
- KI-Argumentationsbasis

---

## 12. Interne musikalische Repräsentationen

### 12.1 Source Graph
Zentrale Repräsentation des Eingangsmaterials.

### Enthält
- BarGrid
- Stems
- ChordTimeline
- BassTimeline
- MelodyContours
- Sections
- LoopCandidates
- SlicePools
- EmbeddingWindows
- ConfidenceMap

### Aufgabe
Der Source Graph trennt das System vom Roh-Audio.  
Ab diesem Punkt arbeitet Riotbox primär mit **musikalischen Zuständen**, nicht mit einer undurchsichtigen Datei.

### 12.2 Scene Graph
Beschreibt die aktuell performbaren Szenen.

### Beispiel
- `intro`
- `reveal`
- `build`
- `strip`
- `slam`
- `breakdown`
- `fake_drop`
- `real_drop`
- `switchup`
- `exit`

Jede Szene definiert:
- welche Quellen aktiv sind
- welche Geräte dominieren
- was geloopt ist
- welche Mutationsbudgets gelten
- ob Ghost-Aktionen erlaubt sind

### 12.3 Phrase Model
Phrasen werden als semantische Bausteine gespeichert.

### Eine Phrase enthält
- Taktlänge
- Startposition
- Trigger-/Note-/Slice-Ereignisse
- Kontur
- Dichte
- Harmoniebezug
- Energielevel
- Herkunft (input / generated / resampled)

### 12.4 Pad Object
Für W-30-artige Sample-Pads.

### Eigenschaften
- audio source
- start/end/loop points
- root note
- pitch mode
- reverse
- grit profile
- envelope
- bank slot
- tags
- provenance

### 12.5 Action Log
Jede relevante Operation wird protokolliert.

### Beispiele
- `capture_loop(L7 -> bank B, pad 3)`
- `generate_202_phrase(scene=build, key=Em)`
- `replace_kick(mode=layered)`
- `ghost_strip_hats(duration=1 bar)`
- `resample_pad(C4 -> D1)`

Das Action Log ist Basis für:
- Undo/Redo
- Replay
- Debugging
- AI-Erklärbarkeit

---

## 13. Geräte-Engine-Design

### 13.1 MC-202 Engine

### Aufgaben
- monophone Basslines erzeugen
- kurze Leads und Antwortphrasen spielen
- Glide/Accent musikalisch einsetzen
- harmonisch folgen oder provozieren
- Sequenzzellen mutieren

### Kernmodule
- Mono Synth Voice
- Phrase Generator
- Step/Cell Sequencer
- Accent/Slide Logic
- Follower/Instigator Logic
- Modulation + FX

### Phrase-Objekt
```text
Step {
  pitch
  gate
  accent
  slide
  octave_flag
  rest
  tie
}
```

### Modi
#### Follower
- folgt Basszentren
- stützt Harmonie
- arbeitet defensiv

#### Instigator
- erzeugt Gegenbewegungen
- übernimmt Führungsrolle
- provoziert Szenenwechsel

### Master-Makro
`202_touch`

Steuert:
- Lautheit/Präsenz
- Accent-Dichte
- Glide-Häufigkeit
- Filter-Drive
- Melodische Eigenständigkeit
- Szenen-Dominanz

### Klangparameter
- cutoff
- resonance
- env amount
- accent amount
- slide time
- pwm depth
- drive
- mono spread (subtil)
- instability / battery drift

### Designregel
Immer **prägnant vor komplex**.  
Die 202 darf nicht alles spielen, sondern das Richtige.

---

### 13.2 W-30 Engine

### Aufgaben
- Slices und Loops in spielbare Bänke überführen
- resamplen
- internen Output zurückführen
- Pad-basiertes Live-Spiel ermöglichen
- spontane Capture-Momente konservieren

### Kernmodule
- Slice Pool
- Loop Miner
- Pad Forge
- Resample Lab
- Bank Manager
- Pitch/Rate Engine
- Grit/Color Engine
- Loop Freezer

### Hauptprinzip
**Capture ist Primäraktion.**  
Jeder gute Moment muss sofort:
- als Pad speicherbar,
- als Loop markierbar,
- als Szene promotbar,
- oder als Resample erneut nutzbar sein.

### Wichtige Operationen
- slice
- auto-pad-map
- trim
- reverse
- repitch
- timestretch (optional, vorsichtig)
- resample internal bus
- granularize (später)
- freeze
- destruct
- bank swap

### Wichtige Makros
- `w30_grit`
- `loop_freeze`
- `resample_now`
- `slice_density`
- `bank_morph`

---

### 13.3 TR-909 Engine

### Aufgaben
- Drums verstärken oder ersetzen
- Pattern auf Basis des analysierten Grooves neu schreiben
- Hats, Claps, Accents und Fills sinnvoll platzieren
- Drops vorbereiten

### Kernmodule
- Pattern Generator
- Drum Reinforcement
- Accent Engine
- Fill Brain
- Groove Quantizer
- Slam Bus

### Arbeitsweise
Die 909-Engine sollte:
- Kick und Snare aus dem Input lesen
- den Groove nicht zerstören
- aber gezielt Punch hinzufügen
- bei Bedarf komplett neu übernehmen

### Modi
- `reinforce`
- `replace`
- `hybrid`
- `skeleton_only`

### Makros
- `909_slam`
- `hat_density`
- `fill_intensity`
- `accent_push`
- `room_amount`

---

## 14. Arrangement und Kompositionslogik

### 14.1 Hierarchischer Aufbau
Riotbox erzeugt Musik nicht auf Pattern-Ebene allein, sondern in drei Schichten:

1. **Section Grammar**
2. **Phrase Generation**
3. **Micro Variation**

### 14.2 Section Grammar
Grundformen:

```text
intro -> reveal -> build -> strip -> slam -> breakdown -> switchup -> final -> exit
```

Nicht jede Session nutzt alle Zustände.  
Das Arrangement-System verwaltet:

- Energie
- Kontrast
- Wiederholung
- Überraschung
- Mutationsbudget
- Originalmaterial-Anteil

### 14.3 Phrase-Generatoren
Jede Lane schreibt Phrasen im Kontext der aktuellen Szene.

### Lanes
- MC-202
- W-30 pads/loops
- TR-909 drums
- FX/transitions
- optional vocal fragments

### 14.4 Micro Variation
Alle paar Takte kleine Mutationen:
- einzelne Slice-Swaps
- Accent-Verschiebung
- Filteröffnung
- halbe Bar Stille
- Ghost Notes
- kurzer Reverse-Call
- Snare Fill
- Pattern-Kompression

**Regel:** Wiedererkennung plus Bewegung.

---

## 15. Scoring und Auswahl

Riotbox generiert nicht blind einen Kandidaten, sondern nutzt **Generate → Score → Select → Mutate**.

### 15.1 Groove Score
Misst:
- Backbeat-Klarheit
- Syncopation-Fitness
- Microtiming-Verträglichkeit
- Tanzbarkeit im Kontext

### 15.2 Identity Score
Misst:
- Erinnerbarkeit
- Konturschärfe
- prägnante Reibung
- rhythmische Signatur

### 15.3 Impact Score
Misst:
- Drop-Tauglichkeit
- Build-Spannung
- Strip-Wirkung
- Abschnittskompatibilität

### 15.4 Novelty Score
Misst:
- Abstand zu den letzten Takten
- Abstand zu konkurrierenden Kandidaten
- internen Similarity-Abstand

### 15.5 Restraint Score
Misst:
- Mix-Überladung
- Frequenzkonflikte
- zu hohe Ornamentik
- Überaktivität des Moments

---

## 16. Live-Mutation und Performance-Steuerung

### 16.1 Grundidee
Riotbox ist live dann stark, wenn Mutationen **hörbar**, **quantisiert**, **verständlich** und **reversibel** sind.

### 16.2 Mutationsarten
- regenerate current scene
- mutate selected lane
- capture current bar group
- strip drums
- slam drums
- swap loop
- instantiate 202 answer
- promote resample
- reverse transition
- fake drop
- restore source

### 16.3 Quantisierung
Mutationen dürfen nur committen auf:
- nächster Beat
- nächste halbe Bar
- nächste Bar
- nächste Phrase
- nächste Szene

Die Standard-Einstellung für sichere Bühne:
- kreative Änderungen: nächste Bar
- harte Rebuilds: nächste Phrase

### 16.4 Undo/Redo
Pflichtfunktionen:
- letzter Commit rückgängig
- Snapshot laden
- Szene wiederherstellen
- Ghost-Aktion revidieren
- Bank-Zustand zurückholen

---

## 17. KI-Agent / Ghost-System

### 17.1 Rolle der KI
Die KI ist **kein direkter Audio-Generator**, sondern ein **Werkzeug-benutzender musikalischer Agent**.

Sie sieht:
- Tempo
- Grid
- Tonart / Akkorde
- aktuelle Szene
- aktive Loops
- verfügbare Pads
- Analysekonfidenz
- Mutationsbudget
- Sperren / Locks
- Verlauf der letzten Aktionen

Sie darf:
- Aktionen planen
- Werkzeuge auslösen
- Vorschläge begründen
- als Ghost performen

### 17.2 Modi
### Watch
- kommentiert nur
- schlägt Chancen vor

### Assist
- schlägt vor
- wartet auf Bestätigung

### Perform
- führt quantisierte Aktionen selbst aus
- respektiert Grenzen und Budgets

### 17.3 Ghost-Protokoll
Beispiel:

```text
[bar 17] ghost: detected strong 2-bar loop candidate from harmonic stem
[bar 21] ghost: generated MC-202 follower phrase in E minor
[bar 25] ghost: stripped hats for 1 bar before drop
[bar 29] ghost: promoted resampled phrase to W-30 bank B, pad 4
```

### 17.4 Sicherheitsgrenzen
Ghost darf niemals:
- Audio-Thread blockieren
- unquantisierte harte Änderungen auslösen
- gelockte Elemente zerstören
- ohne Undo-Möglichkeit agieren
- unendliche Action-Loops erzeugen

### 17.5 Lokales Modell
Die KI wird lokal betrieben. Architektur:

- lokales LLM oder reasoning model für Planung
- nicht-LLM MIR-Modelle für musikalisches Verständnis
- Tool-Calling API
- klar definierter Systemzustand
- kurze, deterministische Tool-Antworten

### 17.6 Warum kein “ein Modell kann alles”
Musikalisches Verstehen entsteht aus:
- Beat-Analyse
- Harmonie
- Struktur
- Slice-Qualität
- Energie
- Kontext

Ein LLM allein ist dafür nicht die beste Quelle.  
Deshalb trennt Riotbox:
- **MIR und Audioanalyse**
- **Agentisches Entscheiden**
- **Realtime-Ausführung**

---

## 18. UX-Strategie

### 18.1 Drei Bedienringe

### Ring 1: Jam
Ziel: sofort musikalisch sein.

Zeigt nur die wichtigsten Makros:
- source ↔ rebuild
- `202_touch`
- `w30_grit`
- `909_slam`
- mutation
- density
- energy
- ghost mode

### Ring 2: Sculpt
Ziel: gezielt bearbeiten.

Seiten:
- MC-202
- W-30 Banks
- Slices
- Loops
- TR-909
- Arrangement
- Mixer

### Ring 3: Lab
Ziel: Analyse und Tiefenkontrolle.

Seiten:
- confidence
- chords
- sections
- embeddings
- providers
- logs
- performance diagnostics

### 18.2 UX-Regeln
- Standard-Screen darf niemals erschlagen
- Tiefe Funktionen nur bei Bedarf
- jeder Makro-Dreh muss hörbar sein
- Logs müssen konkret sein
- Captures müssen sofort auffindbar sein
- Ghost-Aktionen müssen sichtbar bleiben

---

## 19. TUI-Konzept

### 19.1 Hauptseiten
- Jam
- Arrange
- MC202
- W30
- TR909
- Mixer
- Assets
- Export
- Diagnostics

### 19.2 Beispiel-Jam-Screen
```text
Riotbox -- file breaksource.mp3 -- mode hybrid -- seed 90317

[142 BPM | E minor | scene BUILD | energy 0.71 | ghost ASSIST]

SOURCE    retain 43    sections 12   loops 27   confidence 0.84
MC202     touch 58     mode FOLLOW   bite 61    drift 04
W30       grit 49      bank B        pads 16    freeze ON
TR909     slam 63      hats 44       fills 21   hybrid ON
MUTATE    density 52   chaos 18      quant 1 bar
GHOST     ready        next: "promote loop L7"

LOG
[bar 25] captured harmonic loop -> W30 B4
[bar 26] generated 202 answer phrase
[bar 28] stripped kick for half-bar pre-drop
```

### 19.3 Tastatur-Shortcuts
- `space` play/pause
- `tab` page cycle
- `1..8` scene launch / pad bank quick select
- `m` mutate selected lane
- `c` capture
- `l` lock selected object
- `u` undo
- `r` redo / reseed (context-abhängig)
- `g` ghost mode toggle
- `f` fill next bar
- `d` drop next phrase
- `x` destruct selected object
- `s` snapshot save
- `e` export
- `?` help overlay

---

## 20. Controller- und Hardware-Integration

### 20.1 Mindestumfang
- MIDI CC Learn
- Note/Pad Trigger
- Transport
- Bank Selection
- Scene Launch
- Crossfader-artige Makros
- Parameter Feedback, wenn möglich

### 20.2 Primäre Live-Mappings
- source ↔ rebuild
- `202_touch`
- `w30_grit`
- `909_slam`
- mutation
- density
- energy
- ghost aggression

### 20.3 Performance-Philosophie
Ein generisches 8-Knob-Controller-Setup muss bereits musikalisch nutzbar sein.

---

## 21. Sessionmodell

### 21.1 Session enthält
- Projektmetadaten
- Input-Referenzen
- Analyse-Cache-IDs
- globale Seeds
- Lane-Seeds
- Provider-Konfiguration
- Device-Parameter
- Szenenhistorie
- Capture-Bänke
- Action Log
- Ghost-Historie
- Snapshots
- Exporte

### 21.2 Determinismus
Für eine Session mit identischem:
- Input
- Analyse-Cache
- Seed
- Provider-Setup
- Action Log

muss ein reproduzierbarer Rebuild möglich sein.

### 21.3 Snapshot-Typen
- quick snapshot
- scene snapshot
- full session snapshot
- export snapshot

---

## 22. Datenmodell (vereinfachte Skizze)

```text
Session
  id
  metadata
  source_manifest
  analysis_ref
  engine_state
  scene_graph
  bank_state
  controller_map
  action_log
  snapshots

AnalysisBundle
  stems
  bar_grid
  chord_timeline
  contour_data
  sections
  loops
  slices
  embeddings
  confidence

Scene
  id
  type
  energy
  active_lanes
  loop_refs
  pad_refs
  mutation_budget
  locks

Action
  timestamp
  bar_position
  actor(user|ghost|system)
  command
  params
  result
  undo_payload
```

---

## 23. Capture und Looping als Herzstück

### 23.1 Capture-Arten
- Source Loop Capture
- Device Output Capture
- Bus Capture
- Full Scene Capture
- Ghost Favorite Capture

### 23.2 Capture-Workflow
1. guter Moment entsteht
2. Benutzer oder Ghost markiert ihn
3. Quantisierte Extraktion
4. Speicherung als Loop, Pad oder Szene
5. direkte Wiederverwendung

### 23.3 Warum das zentral ist
Das ist der Punkt, an dem Riotbox vom “Generator” zum **musikalischen Werkzeug** wird.  
Nicht nur erzeugen, sondern **Momente ernten und weiterverwenden**.

---

## 24. Resample-Lab

### 24.1 Idee
Riotbox darf nicht nur das Eingangssignal bearbeiten, sondern auch **sich selbst sampeln**.

### 24.2 Beispielkette
1. MC-202 erzeugt gute Phrase
2. Phrase wird intern gebounced
3. W-30 sampelt sie auf Pad
4. Pad wird gepitcht/reverset/gesliced
5. TR-909 übernimmt einen Fill daraus
6. Ghost promoted das Ergebnis in eine neue Szene

### 24.3 Wert
Das schafft eine selbstreferenzielle Kreativitätsschleife und verleiht dem Instrument Eigenleben.

---

## 25. FX- und Mixer-Strategie

### 25.1 Mixer-Layer
- Input stem buses
- MC-202 bus
- W-30 bus
- TR-909 bus
- FX bus
- master bus

### 25.2 Essenzielle Effekte
- drive
- filter
- delay
- room reverb
- compressor/limiter
- bit reduction
- sample rate reduction
- transient shaping
- tape-ish saturation (später optional)

### 25.3 Designregel
Effekte dienen der Performance und der Geräte-Persönlichkeit, nicht der maximalen Studio-Universalität.

---

## 26. Export und Interoperabilität

### 26.1 Export-Arten
- Stereo Mixdown
- Stem Export
- MIDI Export
- Session Export
- Snapshot Export
- Provenance Manifest
- optional Sync/Clock-Referenz

### 26.2 Stems
Mindestens:
- drums
- bass/202
- sampler/w30
- vocals/fragments
- FX
- full mix

### 26.3 Export-Anforderungen
- reproduzierbar
- mit Seed/Session referenzierbar
- klar benannt
- in Batch nutzbar

---

## 27. Qualitätssicherung und Profi-Härtung

### 27.1 Realtime-Stabilität
Pflichtmetriken:
- xruns
- callback timing
- buffer underruns
- CPU peak
- memory growth
- sidecar latency
- action queue lag

### 27.2 Testarten
### Audio/Signal
- voice correctness
- no-click guarantees
- envelope behavior
- scheduler timing

### Logik
- scene transitions
- undo/redo
- capture integrity
- deterministischer replay

### Integration
- audio core ↔ sidecar
- provider swapping
- model timeout handling
- crash recovery

### Golden Renders
Referenz-Renderings für:
- gleiche Seeds
- identische Aktionen
- gleiche Input-Dateien

### 27.3 Crash-Strategie
- Sidecar-Neustart ohne Audio-Stop
- Analysejobs abbrechbar
- Ghost deaktivierbar
- Panic-Funktion für Live-Betrieb

---

## 28. Provider-Architektur

### 28.1 Warum Provider
Viele Analyse- und KI-Bausteine werden sich ändern.  
Daher braucht Riotbox austauschbare Interfaces.

### 28.2 Provider-Typen
- `StemProvider`
- `BeatProvider`
- `HarmonyProvider`
- `ContourProvider`
- `EmbeddingProvider`
- `AgentProvider`
- `StretchProvider`

### 28.3 Vorteil
- schnelle Prototypen
- spätere Lizenzwechsel
- Vergleich mehrerer Methoden
- robustere Produktstrategie

---

## 29. Recht, Lizenzen, Originalität

### 29.1 Grundhaltung
Riotbox soll professionell einsetzbar sein.  
Deshalb ist die rechtliche und ästhetische Trennung von:
- struktureller Inspiration
- materiellem Copying
- rekombiniertem Input

früh mitzudenken.

### 29.2 Maßnahmen
- Betriebsmodi klar trennen
- Provenance auf Objekt-Ebene speichern
- Quellanteile markieren
- Similarity-Firewall als spätere Exportstufe vorsehen
- Provider-Lizenzen isolieren

### 29.3 Similarity-Firewall (spätere Ausbaustufe)
Vor Export kann geprüft werden:
- rhythmische Ähnlichkeit
- melodische Kontur
- Hook-Redundanz
- interne Wiederholungsprobleme

Bei Grenzwerten:
- Phrase mutieren
- andere Loop-Wahl
- Alternativkandidat einsetzen

---

## 30. Repository-Struktur

```text
riotbox/
  README.md
  docs/
    vision.md
    architecture.md
    tui.md
    audio-engine.md
    ai-agent.md
    providers.md
  crates/
    cli/
    core/
    audio/
    dsp/
    devices_mc202/
    devices_w30/
    devices_tr909/
    arranger/
    session/
    ui/
    export/
    ipc/
  python/
    sidecar/
      api/
      providers/
      pipelines/
      scoring/
      agent/
      cache/
  assets/
    presets/
    controller_maps/
    demo_projects/
  tests/
    golden/
    integration/
    property/
  scripts/
  data/
    caches/
    sessions/
    exports/
```

---

## 31. MVP-Definition

### 31.1 Ziel des MVP
Ein Benutzer lädt einen Track und kann innerhalb weniger Minuten:
- die Struktur analysieren,
- einen spielbaren Rebuild starten,
- Loops capturen,
- 202 und 909 live steuern,
- eine KI assistieren lassen,
- das Ergebnis als Session speichern.

### 31.2 Was im MVP drin sein muss
- Audio laden
- grundlegende Analyse
- Source Graph
- einfacher Hybrid-Rebuild
- MC-202 Follower
- W-30 Pad Capture
- TR-909 Reinforcement
- Jam-Screen
- Quantized mutation
- Undo
- lokale Agent-Vorschläge
- Session Save/Load

### 31.3 Was bewusst nicht im MVP ist
- Vollwertige Vocal-Manipulation
- vollständiger DAW-Export-Workflow
- polierter Installer für alle Plattformen
- komplexe granular/spectral Spezialverfahren
- Multi-user / Netzwerkfunktionen
- Cloud-AI
- Plugin-Format

---

## 32. Umsetzungsphasen

### Phase 0 — Sound Bible & Spezifikation
### Ziele
- Klangvision schriftlich fixieren
- Referenzvokabular definieren
- Terminologie vereinheitlichen
- erste TUI-Skizzen
- Action-Lexikon definieren

### Exit-Kriterien
- akzeptiertes Vokabular
- definierte Makros
- definierte Gerätepersönlichkeiten
- definierter MVP

---

### Phase 1 — Core Skeleton
### Inhalt
- Projektstruktur
- Rust Audio Core
- TUI-Grundgerüst
- Transport
- Scheduler
- Session-State
- Action Log
- Snapshot-Grundlagen

### Exit-Kriterien
- stabiles Playback
- UI reagiert
- Sessions lassen sich speichern
- Scheduling ist testbar

---

### Phase 2 — Analyse-Vertical-Slice
### Inhalt
- Datei laden
- Decode/Normalize
- BarGrid
- Sections
- erste Loop-Kandidaten
- rudimentäre Stem-Trennung
- Sidecar-RPC

### Exit-Kriterien
- ein Track wird in Source Graph übersetzt
- Jam-Screen zeigt sinnvolle Analysewerte
- erste quantisierte Loop-Captures möglich

---

### Phase 3 — TR-909 MVP
### Inhalt
- Drum-Detektion
- Reinforcement/Layering
- einfache Pattern-Übernahme
- Accent/Fills
- Drum Bus

### Exit-Kriterien
- Inputdrums lassen sich hörbar stärken
- 909 kann kontrolliert übernehmen
- Fills sind live triggerbar

---

### Phase 4 — MC-202 MVP
### Inhalt
- Mono Synth Voice
- Follower-Bassline
- Accent/Slide
- Phrase-Generator
- `202_touch`

### Exit-Kriterien
- gute follower-Basslines
- live steuerbare Klangparameter
- quantisierte Phrase-Mutation

---

### Phase 5 — W-30 MVP
### Inhalt
- Slice Pool
- Pad Forge
- Loop Freezer
- Bank Manager
- Resample Lab v1

### Exit-Kriterien
- gute Loops können gecaptured werden
- Pads sind spielbar
- interner Bus kann resampled werden

---

### Phase 6 — Scene Brain
### Inhalt
- Scene Graph
- Energie-Management
- Arrangement-Regeln
- Strip/Build/Slam-Logik
- Launch und Restore

### Exit-Kriterien
- aus einem Track entstehen mehrere brauchbare Szenen
- Szenenwechsel klingen musikalisch

---

### Phase 7 — Ghost / AI Assist
### Inhalt
- Tool API
- lokaler Agent
- Watch/Assist/Perform
- Ghost Log
- Limits und Mutationsbudget

### Exit-Kriterien
- Agent kann sinnvolle Vorschläge machen
- Agent kann quantisierte Aktionen sicher ausführen
- alles bleibt undo-bar

---

### Phase 8 — Pro Hardening
### Inhalt
- Robustheit
- Diagnose
- Crash-Recovery
- deterministischer Replay
- Export
- Provider-Austauschbarkeit

### Exit-Kriterien
- bühnentauglicher Durchlauf
- Session-Replay verlässlich
- Exporte reproduzierbar

---

## 33. Detaillierter Backlog nach Modulen

### 33.1 Audio Core
- [ ] Device-Auswahl
- [ ] Sample-Rate-Management
- [ ] Buffer-Management
- [ ] Scheduler
- [ ] Voice Pool
- [ ] Mixer
- [ ] FX sends
- [ ] master limiter
- [ ] panic state

### 33.2 Session / State
- [ ] Session-ID
- [ ] Serialization
- [ ] Snapshot-System
- [ ] Action Log
- [ ] undo payloads
- [ ] replay engine
- [ ] version migration

### 33.3 Analysis Sidecar
- [ ] RPC server
- [ ] job queue
- [ ] cache store
- [ ] decode pipeline
- [ ] beat/bar analysis
- [ ] harmony analysis
- [ ] contour extraction
- [ ] loop miner
- [ ] embedding interface

### 33.4 MC-202
- [ ] mono oscillator core
- [ ] filter
- [ ] envelopes
- [ ] accent model
- [ ] slide model
- [ ] follower generator
- [ ] instigator generator
- [ ] phrase scoring
- [ ] UI page

### 33.5 W-30
- [ ] slice editor
- [ ] pad mapping
- [ ] bank switching
- [ ] resample routing
- [ ] reverse/pitch/rate
- [ ] capture shortcuts
- [ ] provenance tracking

### 33.6 TR-909
- [ ] kick layer
- [ ] snare layer
- [ ] hat generator
- [ ] accent lane
- [ ] fill generator
- [ ] pattern modes
- [ ] slam bus

### 33.7 Arrangement
- [ ] section classifier
- [ ] scene templates
- [ ] energy model
- [ ] mutation rules
- [ ] drop preparation
- [ ] restore rules

### 33.8 TUI
- [ ] Jam screen
- [ ] MC202 page
- [ ] W30 page
- [ ] TR909 page
- [ ] Arrange page
- [ ] mixer page
- [ ] log panel
- [ ] help overlay
- [ ] performance meters

### 33.9 AI Agent
- [ ] state summarizer
- [ ] tool schema
- [ ] budget manager
- [ ] watch mode
- [ ] assist mode
- [ ] perform mode
- [ ] explanation formatter
- [ ] guard rails

---

## 34. Nichtfunktionale Anforderungen

### 34.1 Latenz
- spielbare Reaktion für Makros
- quantisierte Commit-Logik
- klar sichtbare “pending action”-Zustände

### 34.2 Stabilität
- Audio darf nicht knacksen
- Ghost darf deaktivierbar sein
- Analyse darf ausfallen, Session darf weiterlaufen

### 34.3 Speicher und CPU
- konservative Echtzeitpfade
- große Samples streamen oder vorhalten
- Sidecar-Last begrenzen
- graceful degradation

### 34.4 Bedienbarkeit
- sinnvolle Defaults
- starke Presets
- gutes Logging
- keine Parameterflut im Jam-Screen

---

## 35. Preset- und Style-System

### 35.1 Preset-Ebenen
- globales Projektpreset
- Gerätepreset
- Szenenpreset
- Controller-Mapping
- Ghost-Policy

### 35.2 Erste Stilfamilien
- `feral_break`
- `acid_wire`
- `sampler_bruise`
- `night_slam`
- `ghost_cut`

Jede Familie definiert:
- Energieverhalten
- Device-Gewichte
- Mutationsfreude
- Quelltreue
- Grit/Drive
- Ghost-Autonomie

---

## 36. Preset-Makros

### Globale Makros
- `source_retain`
- `energy`
- `mutation`
- `density`
- `ghost`

### Geräte-Makros
- `202_touch`
- `w30_grit`
- `loop_freeze`
- `909_slam`
- `hat_density`
- `bank_morph`

### Performance-Makros
- `fake_drop`
- `destroy_rebuild`
- `promote_now`
- `restore_source`
- `capture_now`

---

## 37. Explainability und Vertrauen

### 37.1 Warum wichtig
Sobald KI und komplexe Analyse mitspielen, braucht der Benutzer Vertrauen.

### 37.2 Maßnahmen
- jede Ghost-Aktion wird geloggt
- wichtige Analysewerte sichtbar
- Confidence-Werte anzeigen
- “warum diese Aktion?” erklärbar
- Locks und Budgets sichtbar
- Undo immer in Reichweite

---

## 38. Risikoanalyse

### 38.1 Technische Risiken
### Realtime vs. KI
Gefahr: Modellaufrufe blockieren indirekt den Flow.  
Gegenmaßnahme: strikte Prozessgrenzen.

### Analysequalität
Gefahr: schlechte Beat-/Chord-Erkennung.  
Gegenmaßnahme: alternative Hypothesen, Confidence und manuelle Korrekturpfade.

### Überkomplexität
Gefahr: zu viele Features zu früh.  
Gegenmaßnahme: klarer MVP und harte Priorisierung.

### Unmusikalische Mutation
Gefahr: System klingt clever, aber nicht gut.  
Gegenmaßnahme: Scoring, Capture-first, starke Presets, reale Test-Sessions.

### 38.2 Produkt-Risiken
### UX drift
Gefahr: Tool wird Nerd-Labor statt Instrument.  
Gegenmaßnahme: Jam-first.

### Stilklischee
Gefahr: zu plakative Retro-Kopie.  
Gegenmaßnahme: Geräte als Rollen, nicht als Cosplay.

### Lizenz-/Provider-Probleme
Gefahr: unpassende Analyse- oder Modelllizenzen.  
Gegenmaßnahme: Provider-Schnittstellen und saubere Austauschbarkeit.

---

## 39. Team- und Rollenempfehlung

Für effiziente Entwicklung sinnvoll:

- **Product / Sound Director**  
  hält Vision, Sound-Bible und UX-Schärfe

- **Realtime Audio Engineer**  
  Audio Core, DSP, Scheduling, Gerätelogik

- **MIR / ML Engineer**  
  Analysepipeline, Features, Scoring, Agent-Integration

- **TUI / Interaction Engineer**  
  Jam/Sculpt/Lab, Controller-Mappings, Diagnose

- **QA / Music Tester**  
  reale Session-Tests, Golden Renders, Klangkritik

In der Frühphase können Rollen kombiniert werden.

---

## 40. Erste Meilensteine in Wochenlogik

## Meilenstein A
**“Track laden, sehen, loopen.”**
- Datei laden
- Analyse starten
- Grid/Sections anzeigen
- Top-Loops capturen

## Meilenstein B
**“Track mutieren, aber musikalisch.”**
- 909-Reinforcement
- 202-Follower
- einfache Szenen

## Meilenstein C
**“Track wird Instrument.”**
- W-30-Pads
- Capture-first Workflow
- Resampling

## Meilenstein D
**“Instrument bekommt Eigenleben.”**
- Ghost Watch/Assist
- Actions, Budgets, Logs

## Meilenstein E
**“Bühnentauglich.”**
- Pro Hardening
- Replay
- Export
- Crash-Sicherheit

---

## 41. Entscheidungsregeln für die Entwicklung

Wenn zwischen zwei Varianten gewählt wird, gilt:

1. **Stabilität vor Cleverness**
2. **Musikalische Wirkung vor algorithmischer Eleganz**
3. **Instrumenten-Charakter vor Feature-Masse**
4. **Capture und Performance vor Offline-Perfektion**
5. **Nachvollziehbarkeit vor Magie**
6. **MVP-Schärfe vor Konzeptinflation**

---

## 42. Schlussbild

Riotbox soll sich anfühlen wie:

- ein **wilder Hardware-Hybrid**, der nie gebaut wurde,
- eine **feral** Mischung aus **MC-202**, **W-30** und **TR-909**,
- ein **terminalbasiertes Live-Instrument**, kein verkleidetes Forschungsprojekt,
- ein System, das aus Audio **Momente**, aus Momenten **Phrasen** und aus Phrasen **neue Szenen** macht,
- ein Werkzeug, das man **spielt, beobachtet, erntet und wieder anheizt**.

Der Kern des Produkts ist nicht “KI macht Musik”, sondern:

> **Riotbox übersetzt Klang in musikalische Handlungsräume.**  
> Der Mensch, der Ghost oder beide zusammen bespielen diese Räume live.

---

## 43. Nächste konkrete Dokumente

Nach diesem Masterplan sollten als nächste Artefakte entstehen:

1. **PRD v1**  
   exakte Produktanforderungen für MVP

2. **Audio Core Spec**  
   Scheduler, Mixer, Voices, Timing, FX, Buffers

3. **Source Graph Spec**  
   Datenmodell und Analyse-Outputs

4. **TUI Screen Spec**  
   Seiten, Zustände, Tastatur und Makros

5. **Ghost API Spec**  
   Tool-Schema, Budgets, Aktionen, Logs

6. **Session File Spec**  
   Serialisierung, Migration, Replay

7. **Preset & Style Spec**  
   Makrobereiche, Stilfamilien, Gerätegewichte

---

## 44. Ein-Satz-Version

> **Riotbox ist ein ferales Terminal-Audioinstrument, das eingehendes Audio in ein spielbares, mutierbares und KI-steuerbares Live-System aus MC-202-, W-30- und TR-909-Persönlichkeiten übersetzt.**
