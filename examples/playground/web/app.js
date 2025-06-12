import { default as createModule } from "./playground.js";

// -------------------------------------------------------------------------------------------------

const defaultBpm = 120;
const defaultInstrument = 0;
const defaultScriptContent = `--
-- Welcome to the Renoise pattrns playground!
--
-- Create and experiment with pattern scripts here to learn how the work.
-- Check out the interactive 'Quickstart' scripts on the right, or load some examples
-- to get started.
--
-- This playground uses a simple sample player as the player backend. The currently 
-- selected sample plays by default, unless your script specifies a different instrument
-- explicitly.
--
-- Use \`Control + Return\` in the editor to *apply* changes you've made to the script. 
-- Use \`Control + Shift + Space\` to start stop playing.

local chord = { "c4", "e4", "g4", "a4", "e4", "a3" } -- The arpeggio notes
local length = 32 -- Modulation length in units

return pattern {
  unit = "1/16",
  event = function(context)
    -- Cycle through chord notes
    local step = math.imod(context.step, #chord)
    -- Return the note event
    return {
        key = chord[step],
        -- Create a volume swell using cosine wave
        volume = 0.6 + 0.4 * math.cos(context.step / length * math.pi),
        -- Add stereo movement with sine wave panning
        panning = 0.5 * math.sin(context.step / length / 3 * math.pi),
        -- Set the instrument (change to 0 for bass sample, nil plays the selected one)
        instrument = nil
    }
  end
}`;

// -------------------------------------------------------------------------------------------------

var backend = {
    _playground: undefined,
    _isPlaying: false,

    initialize: function (playground) {
        this._playground = playground;

        const err = this._playground.ccall('initialize_playground', 'string', [])
        if (err) {
            return err
        }

        this.updateBpm(defaultBpm);
        this.updateInstrument(defaultInstrument);
        this.updateScriptContent(defaultScriptContent);

        return undefined;
    },

    getSamples: function () {
        const ptr = this._playground.ccall('get_samples', 'number', [])
        const samplesJson = this._playground.UTF8ToString(ptr);
        this._playground.ccall('free_cstring', 'undefined', ['number'], [ptr])
        return JSON.parse(samplesJson);
    },

    getQuickstartScripts: function () {
        const ptr = this._playground.ccall('get_quickstart_scripts', 'number', [])
        const examplesJson = this._playground.UTF8ToString(ptr);
        this._playground.ccall('free_cstring', 'undefined', ['number'], [ptr])
        return JSON.parse(examplesJson);
    },

    getExampleScripts: function () {
        const ptr = this._playground.ccall('get_example_scripts', 'number', [])
        const examplesJson = this._playground.UTF8ToString(ptr);
        this._playground.ccall('free_cstring', 'undefined', ['number'], [ptr])
        return JSON.parse(examplesJson);
    },

    getScriptError: function () {
        let ptr = this._playground.ccall('get_script_error', 'number', [])
        const err = this._playground.UTF8ToString(ptr);
        this._playground._free_cstring(ptr);
        return err;
    },

    isPlaying: function () {
        return this._isPlaying;
    },

    startPlaying: function () {
        this._playground.ccall("start_playing");
        this._isPlaying = true;
    },

    stopPlaying: function () {
        this._playground.ccall("stop_playing");
        this._isPlaying = false;
    },

    stopPlayingNotes: function () {
        this._playground.ccall("stop_playing_notes");
    },

    sendMidiNoteOn: function (note, velocity) {
        this._playground.ccall("midi_note_on", 'undefined', ['number', 'number'], [note, velocity]);
    },

    sendMidiNoteOff: function (note) {
        this._playground.ccall("midi_note_off", 'undefined', ['number'], [note]);
    },

    updateInstrument: function (instrument) {
        this._playground.ccall("set_instrument", 'undefined', ['number'], [instrument]);
    },

    updateBpm: function (bpm) {
        this._playground.ccall("set_bpm", 'undefined', ['number'], [bpm]);
    },

    updateScriptContent: function (content) {
        this._playground.ccall("update_script", 'undefined', ['string'], [content]);
    },
};

// -------------------------------------------------------------------------------------------------

var app = {
    _initialized: false,
    _editor: undefined,
    _editCount: 0,

    initialize: function () {
        // hide spinner, show content
        document.getElementById('loading').style.display = 'none';
        document.getElementById('content').style.display = 'flex';
        this._initialized = true;

        // init components
        this._initControls();
        this._initSampleDropdown();
        this._initExampleScripts();
        this._initScriptErrorTimer();
        this._initEditor();
    },

    // Show status message in loading screen or status bar
    setStatus: function (message, isError) {
        // log to console
        (isError ? console.error : console.log)(message);
        // update app text
        const statusElement = this._initialized
            ? document.getElementById('status')
            : document.getElementById('spinner-status');
        if (statusElement != undefined) {
            statusElement.textContent = message.replace(/(?:\r\n|\r|\n)/g, '\t');
            statusElement.style.color = isError ? 'var(--color-error)' : 'var(--color-success)';
            // clear non-error messages after 5 seconds
            if (this._clearStatusTimeout) {
                clearTimeout(this._clearStatusTimeout);
                this._clearStatusTimeout = null;
            }
            if (!isError) {
                this._clearStatusTimeout = setTimeout(() => {
                    statusElement.textContent = '';
                }, 5000);
            }
        }
        else {

        }
    },

    // Init transport controls
    _initControls: function () {
        // Set up control handlers
        const playButton = document.getElementById('playButton');
        const stopButton = document.getElementById('stopButton');
        const midiButton = document.getElementById('midiButton');
        console.assert(playButton && stopButton && midiButton);

        playButton.addEventListener('click', () => {
            backend.startPlaying();
            this.setStatus("Playing...");
            playButton.style.color = 'var(--color-accent)';
        });

        stopButton.addEventListener('click', () => {
            backend.stopPlaying();
            this.setStatus("Stopped");
            playButton.style.color = null;
        });

        const bpmInput = document.getElementById('bpmInput');
        console.assert(bpmInput);

        bpmInput.addEventListener('change', (e) => {
            const bpm = parseInt(e.target.value);
            if (!isNaN(bpm)) {
                backend.updateBpm(bpm);
                this.setStatus(`Set new BPM: '${bpm}'`);
            }
        });

        let midiAccess = null;
        let midiEnabled = false;
        let currentMidiNotes = new Set();

        function enableMidi() {
            if (!navigator.requestMIDIAccess) {
                return Promise.reject(new Error("Web MIDI API not supported"));
            }
            return navigator.requestMIDIAccess()
                .then(access => {
                    midiAccess = access;
                    midiEnabled = true;
                    midiButton.style.color = 'var(--color-accent)';
                    // Start listening to MIDI input
                    for (let input of midiAccess.inputs.values()) {
                        input.onmidimessage = handleMidiMessage;
                    }
                    // stop regular playback
                    if (backend.isPlaying()) {
                        backend.stopPlaying();
                        playButton.style.color = null;
                    }
                    app.setStatus("MIDI input enabled. Press one or more notes on your keyboard to play the script...");
                });
        }

        function disableMidi() {
            midiEnabled = false;
            midiButton.style.color = null;
            // Stop listening to MIDI input
            if (midiAccess) {
                for (let input of midiAccess.inputs.values()) {
                    input.onmidimessage = null;
                }
            }
            // Release all notes
            currentMidiNotes.forEach(note => {
                backend.sendMidiNoteOff(note);
            });
            currentMidiNotes.clear();
            app.setStatus("MIDI input disabled");
            return Promise.resolve();
        }

        function handleMidiMessage(message) {
            const data = message.data;
            const status = data[0] & 0xF0;
            const note = data[1];
            const velocity = data[2];
            if (status === 0x90 && velocity > 0) { // Note on
                if (!currentMidiNotes.has(note)) {
                    currentMidiNotes.add(note);
                    backend.sendMidiNoteOn(note, velocity);
                }
            } else if (status === 0x80 || (status === 0x90 && velocity === 0)) { // Note off
                if (currentMidiNotes.has(note)) {
                    currentMidiNotes.delete(note);
                    backend.sendMidiNoteOff(note);
                }
            }
        }

        midiButton.addEventListener('click', () => {
            if (!midiEnabled) {
                enableMidi().then(() => {
                    // Disable play/stop buttons on success
                    playButton.disabled = true;
                    stopButton.disabled = true;
                }).catch(err => {
                    const isError = true;
                    app.setStatus("Failed to access MIDI: " + err, isError);
                });
            } else {
                disableMidi().then(() => {
                    // Re-enable play/stop buttons
                    playButton.disabled = false;
                    stopButton.disabled = false;
                }).catch(err => {
                    const isError = true;
                    app.setStatus("Failed to release MIDI: " + err, isError);
                });
            }
        });
    },

    // Populate sample dropdown
    _initSampleDropdown: function () {
        const samples = backend.getSamples();

        const select = document.getElementById('sampleSelect');
        console.assert(select);

        select.innerHTML = '';
        samples.forEach((sample, index) => {
            const option = document.createElement('option');
            option.value = sample.id;
            option.textContent = `${String(index).padStart(2, '0')}: ${sample.name}`;
            select.appendChild(option);
        });
        select.onchange = (event) => {
            let id = event.target.value;
            backend.updateInstrument(Number(id));
            this.setStatus(`Set new default instrument: '${id}'`);

        };

        // set last sample as default instrument
        select.value = samples[samples.length - 1].id
        backend.updateInstrument(select.value)
    },

    // Set up example scripts list
    _initExampleScripts: function () {
        const examples = backend.getExampleScripts();
        const quickstartExamples = backend.getQuickstartScripts();

        const examplesList = document.getElementById('examples-list');
        examplesList.innerHTML = '';

        // Add quickstart examples
        const quickstartSection = document.createElement('h3');
        quickstartSection.textContent = "Quickstart";
        examplesList.appendChild(quickstartSection);

        let allLinks = [];
        let loadExample = (link, example) => {
            allLinks.forEach(link => {
                link.style.textDecoration = 'none';
            });
            link.style.textDecoration = 'underline';
            this._editor.setValue(example.content);
            if (backend.isPlaying()) {
                backend.stopPlaying();
                backend.updateScriptContent(example.content);
                backend.startPlaying();
            } else {
                backend.updateScriptContent(example.content);
            }
            this._editor.setScrollPosition({ scrollTop: 0 });
            this._updateEditCount(0);
            this.setStatus(`Loaded script: '${example.name}'.`);
        };

        quickstartExamples.forEach(group => {
            const quickstartGroup = document.createElement('h4');
            quickstartGroup.textContent = group.name;
            examplesList.appendChild(quickstartGroup);

            group.entries.forEach(example => {
                const li = document.createElement('li');
                const a = document.createElement('a');
                a.href = '#';
                a.textContent = example.name;
                a.style.color = 'var(--color-link)';
                a.style.textDecoration = 'none';
                a.onclick = () => loadExample(a, example);
                allLinks.push(a)
                li.appendChild(a);
                examplesList.appendChild(li);
            });
        });

        // Add examples
        const examplesSection = document.createElement('h3');
        examplesSection.textContent = "Examples";
        examplesList.appendChild(examplesSection);

        examples.forEach(example => {
            const li = document.createElement('li');
            const a = document.createElement('a');
            a.href = '#';
            a.textContent = example.name;
            a.style.color = 'var(--color-link)';
            a.style.textDecoration = 'none';
            a.onclick = () => loadExample(a, example);
            allLinks.push(a)
            li.appendChild(a);
            examplesList.appendChild(li);
        });
    },

    // Initialize Monaco editor
    _initEditor: function () {
        require.config({ paths: { 'vs': 'https://cdnjs.cloudflare.com/ajax/libs/monaco-editor/0.52.2/min/vs' } });

        let editorElement = document.getElementById('editor');
        console.assert(editorElement);

        require(['vs/editor/editor.main'], () => {
            // Create editor
            this._editor = monaco.editor.create(editorElement, {
                value: defaultScriptContent,
                language: 'lua',
                theme: 'vs-dark',
                minimap: { enabled: false },
                scrollBeyondLastLine: false,
                automaticLayout: true,
                wordWrap: 'on',
                acceptSuggestionOnCommitCharacter: true
            });
            // Track edits
            this._editor.onDidChangeModelContent(() => {
                this._updateEditCount(this._editCount + 1)
            });
            // Handle Ctrl+Enter
            const commitAction = {
                id: "Apply Script Changes",
                label: "Apply Script Changes",
                contextMenuOrder: 0,
                contextMenuGroupId: "script",
                keybindings: [
                    monaco.KeyMod.CtrlCmd | monaco.KeyCode.Enter,
                    monaco.KeyMod.CtrlCmd | monaco.KeyCode.Key_S,
                ],
                run: () => {
                    backend.updateScriptContent(this._editor.getValue());
                    this._updateEditCount(0);
                    this.setStatus("Applied script changes.");
                },
            }
            this._editor.addAction(commitAction);

            // Handle Ctrl+Shift+Space
            const playStopAction = {
                id: "Start/Stop Playback",
                label: "Start/Stop Playback",
                contextMenuOrder: 1,
                contextMenuGroupId: "script",
                keybindings: [
                    monaco.KeyMod.CtrlCmd | monaco.KeyMod.Shift | monaco.KeyCode.Space,
                ],
                run: () => {
                    const playButton = document.getElementById('playButton');
                    if (!playButton.disabled) {
                        if (backend.isPlaying()) {
                            backend.stopPlaying();
                            playButton.style.color = null;
                        }
                        else {
                            backend.startPlaying();
                            playButton.style.color = 'var(--color-accent)';
                        }
                    }
                },
            }
            this._editor.addAction(playStopAction);

            // HACK: don't let the browser handle Control + S 
            document.addEventListener('keydown', e => {
                if (e.ctrlKey && e.key === 's') {
                    // Prevent the Save dialog to open
                    e.preventDefault();
                }
            });

            /*
            // TODO: Register a simple autocomplete provider for Lua for `pattern`
            monaco.languages.registerCompletionItemProvider('lua', {
                provideCompletionItems: function (model, position) {
                    const lineContent = model.getLineContent(position.lineNumber);
                    const textUntilPosition = model.getValueInRange({
                        startLineNumber: 1,
                        startColumn: 1,
                        endLineNumber: position.lineNumber,
                        endColumn: position.column
                    });
    
                    let insidePatternTable = false;
                    let braceDepth = 0;
                    let inPattern = false;
                    for (let i = 0; i < textUntilPosition.length; i++) {
                        const char = textUntilPosition[i];
    
                        if (textUntilPosition.substr(i, 6) === 'pattern') {
                            // Look ahead for opening brace
                            for (let j = i + 6; j < textUntilPosition.length; j++) {
                                if (textUntilPosition[j] === '{') {
                                    inPattern = true;
                                    braceDepth = 1;
                                    i = j;
                                    break;
                                } else if (textUntilPosition[j] !== ' ' && textUntilPosition[j] !== '\t' && textUntilPosition[j] !== '\n') {
                                    break;
                                }
                            }
                        } else if (inPattern) {
                            if (char === '{') {
                                braceDepth++;
                            } else if (char === '}') {
                                braceDepth--;
                                if (braceDepth === 0) {
                                    inPattern = false;
                                }
                            }
                        }
                    }

                    insidePatternTable = inPattern && braceDepth > 0;
                    if (insidePatternTable) {
                        const word = model.getWordUntilPosition(position);
                        const range = {
                            startLineNumber: position.lineNumber,
                            endLineNumber: position.lineNumber,
                            startColumn: word.startColumn,
                            endColumn: word.endColumn
                        };
                        return {
                            suggestions: [
                                {
                                    label: 'event',
                                    kind: monaco.languages.CompletionItemKind.Property,
                                    insertText: 'event = ',
                                    range: range,
                                    sortText: '1'
                                },
                                {
                                    label: 'pulse',
                                    kind: monaco.languages.CompletionItemKind.Property,
                                    insertText: 'pulse = ',
                                    range: range,
                                    sortText: '2'
                                }
                            ]
                        };
                    }
    
                    return { suggestions: [] };
                }
            });
            */
        });
    },

    _initScriptErrorTimer: function () {
        const errorPane = document.getElementById('editor-error');
        console.assert(errorPane);

        const errorContent = document.getElementById('editor-error-content');
        console.assert(errorContent);

        let lastScriptError = "";
        setInterval(() => {
            const err = backend.getScriptError();
            if (err !== lastScriptError) {
                lastScriptError = err;

                // Clear previous markers
                if (this._editor) {
                    monaco.editor.setModelMarkers(
                        this._editor.getModel(),
                        'owner',
                        []
                    );
                }

                if (err) {
                    errorContent.textContent = err;
                    errorPane.style.display = 'flex';

                    // Parse error and add to editor
                    const parsedError = this._parseLuaError(err);
                    if (parsedError && this._editor) {
                        monaco.editor.setModelMarkers(
                            this._editor.getModel(),
                            'owner',
                            [{
                                severity: monaco.MarkerSeverity.Error,
                                message: parsedError.message,
                                startLineNumber: parsedError.lineNumber,
                                startColumn: 1,
                                endLineNumber: parsedError.lineNumber,
                                endColumn: 100 // arbitrary large column
                            }]
                        );
                    }
                } else {
                    errorContent.textContent = '';
                    errorPane.style.display = 'none';
                }
            }
        }, 200);
    },

    // Show hide the "X edits" text
    _updateEditCount: function (count) {
        this._editCount = count;

        const editorStatusContent = document.getElementById('editor-status-content');
        const editCountSpan = document.getElementById('editCount');
        console.assert(editorStatusContent && editCountSpan);

        if (count > 0) {
            editCountSpan.textContent = `${count} edit${count === 1 ? '' : 's'}`;
            editorStatusContent.classList.remove('hidden');
            editorStatusContent.style.backgroundColor = 'var(--color-grid)';
        }
        else {
            editorStatusContent.classList.add('hidden');
            editorStatusContent.style.backgroundColor = 'unset'
        }
    },

    // helper function to get line info from Lua errors
    _parseLuaError: function (error) {
        // Parse Lua error format like: [string "buffer"]:3: 'then' expected near '='
        const match = error.match(/\[string ".*"\]:(\d+):\s*(.*)/);
        if (match) {
            return {
                lineNumber: parseInt(match[1]),
                message: match[2]
            };
        }
        return null;
    }
}

// -------------------------------------------------------------------------------------------------

const webAssemblySupported = (() => {
    try {
        if (typeof WebAssembly === "object" && typeof WebAssembly.instantiate === "function") {
            const module = new WebAssembly.Module(
                Uint8Array.of(0x0, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00));
            if (module instanceof WebAssembly.Module) {
                return new WebAssembly.Instance(module) instanceof WebAssembly.Instance;
            }
        }
    }
    catch (e) {
        // ignore
    }
    return false;
})();

if (webAssemblySupported) {
    let Module = {
        print: (...args) => {
            let isError = false;
            app.setStatus(args.join(' '), isError)
        },
        printErr: (...args) => {
            let isError = true;
            app.setStatus(args.join(' '), isError)
        }
    }

    createModule(Module)
        .then((module) => {
            // initialize backend
            let err = backend.initialize(module);
            if (err) {
                const isError = true;
                app.setStatus(err, true);
            }
            else {
                // initialize app
                app.initialize();
                app.setStatus("Ready");
            }
        }).catch((err) => {
            let isError = true;
            app.setStatus(err.message || "WASM failed to load", isError);
        });

    // redirect global errors
    window.addEventListener("unhandledrejection", function (event) {
        let isError = true;
        app.setStatus(event.reason, isError);
    });
    window.onerror = (message, filename, lineno, colno, error) => {
        let isError = true;
        app.setStatus(message || "Unknown window error", isError);
    };

}
else {
    const isError = true;
    app.setStatus("This page requires WebAssembly support, " +
        "which appears to be unavailable in this browser.", isError);

    let spinner = document.getElementById('spinner');
    if (spinner) {
        spinner.style.display = "None";
    }
}
