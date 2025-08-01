import 'package:flutter/material.dart';
import 'dart:async';

void main() {
  runApp(const FlexMetroApp());
}

class FlexMetroApp extends StatelessWidget {
  const FlexMetroApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'FlexMetro Musical Timer',
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(seedColor: Colors.deepPurple),
        useMaterial3: true,
      ),
      home: const MusicalTimerPage(),
    );
  }
}

class MusicalTimerPage extends StatefulWidget {
  const MusicalTimerPage({super.key});

  @override
  State<MusicalTimerPage> createState() => _MusicalTimerPageState();
}

class _MusicalTimerPageState extends State<MusicalTimerPage>
    with TickerProviderStateMixin {
  late AnimationController _circle1Controller;
  late AnimationController _circle2Controller;
  late AnimationController _circle3Controller;
  
  Timer? _musicalTimer;
  bool _isRunning = false;
  int _currentBeat = 0;
  int _currentBar = 1;
  String _currentTimeSignature = "4/4";
  double _currentTempo = 60.0;
  String _currentBeatType = "MAJOR";
  
  // Musical section: 4/4, 6/8, 4/4 (matching our Rust test)
  final List<Map<String, dynamic>> _musicalSection = [
    {"nom": 4, "denom": 4, "beats": 4},
    {"nom": 6, "denom": 8, "beats": 6}, 
    {"nom": 4, "denom": 4, "beats": 4},
  ];
  
  int _totalBeats = 14; // 4+4+6 = 14 total beats
  double _startTempo = 60.0;
  double _endTempo = 90.0;
  
  @override
  void initState() {
    super.initState();
    
    // Initialize animation controllers for each circle
    _circle1Controller = AnimationController(
      duration: const Duration(milliseconds: 200),
      vsync: this,
    );
    _circle2Controller = AnimationController(
      duration: const Duration(milliseconds: 200),
      vsync: this,
    );
    _circle3Controller = AnimationController(
      duration: const Duration(milliseconds: 200),
      vsync: this,
    );
  }

  @override
  void dispose() {
    _circle1Controller.dispose();
    _circle2Controller.dispose();
    _circle3Controller.dispose();
    _musicalTimer?.cancel();
    super.dispose();
  }

  void _startTimer() {
    if (_isRunning) return;
    
    setState(() {
      _isRunning = true;
      _currentBeat = 0;
      _currentBar = 1;
    });
    
    _scheduleNextBeat();
  }

  void _stopTimer() {
    setState(() {
      _isRunning = false;
    });
    _musicalTimer?.cancel();
    
    // Reset all circles
    _circle1Controller.reset();
    _circle2Controller.reset();
    _circle3Controller.reset();
  }

  void _scheduleNextBeat() {
    if (!_isRunning || _currentBeat >= _totalBeats) {
      _stopTimer();
      return;
    }
    
    // Calculate current tempo based on progress (linear interpolation)
    double progress = _currentBeat / (_totalBeats - 1);
    _currentTempo = _startTempo + (_endTempo - _startTempo) * progress;
    
    // Calculate which bar we're in and the beat type
    int beatsInCurrentBar = 0;
    int barIndex = 0;
    int beatInBar = 0;
    
    for (int i = 0; i < _musicalSection.length; i++) {
      int barBeats = _musicalSection[i]["beats"];
      if (_currentBeat < beatsInCurrentBar + barBeats) {
        barIndex = i;
        beatInBar = _currentBeat - beatsInCurrentBar;
        break;
      }
      beatsInCurrentBar += barBeats;
    }
    
    _currentBar = barIndex + 1;
    _currentTimeSignature = "${_musicalSection[barIndex]["nom"]}/${_musicalSection[barIndex]["denom"]}";
    
    // Determine beat type based on position in bar
    if (beatInBar % 2 == 0) {
      if (beatInBar == 0) {
        _currentBeatType = "MAJOR";
      } else {
        _currentBeatType = "MEDIUM";
      }
    } else {
      _currentBeatType = "MINOR";
    }
    
    _triggerBeat();
    
    // Calculate next beat interval (60000ms / tempo for quarter notes)
    double intervalMs = 60000.0 / _currentTempo;
    if (_currentTimeSignature == "6/8") {
      intervalMs /= 2; // Eighth notes in 6/8
    }
    
    _currentBeat++;
    
    _musicalTimer = Timer(Duration(milliseconds: intervalMs.round()), _scheduleNextBeat);
  }

  void _triggerBeat() {
    // Animate circles based on beat type
    switch (_currentBeatType) {
      case "MAJOR":
        _circle1Controller.forward().then((_) => _circle1Controller.reverse());
        break;
      case "MEDIUM":
        _circle2Controller.forward().then((_) => _circle2Controller.reverse());
        break;
      case "MINOR":
        _circle3Controller.forward().then((_) => _circle3Controller.reverse());
        break;
    }
    
    setState(() {}); // Update UI with new values
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        backgroundColor: Theme.of(context).colorScheme.inversePrimary,
        title: const Text('FlexMetro Musical Timer'),
      ),
      body: Padding(
        padding: const EdgeInsets.all(16.0),
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            // Musical information display
            Card(
              child: Padding(
                padding: const EdgeInsets.all(16.0),
                child: Column(
                  children: [
                    Text(
                      'Musical Section: 4/4 → 6/8 → 4/4',
                      style: Theme.of(context).textTheme.headlineSmall,
                    ),
                    const SizedBox(height: 8),
                    Text(
                      'Tempo: ${_startTempo.toInt()} BPM → ${_endTempo.toInt()} BPM',
                      style: Theme.of(context).textTheme.titleMedium,
                    ),
                  ],
                ),
              ),
            ),
            
            const SizedBox(height: 32),
            
            // Three circles display
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceEvenly,
              children: [
                _buildCircle(
                  color: Colors.yellow,
                  label: "MAJOR",
                  controller: _circle1Controller,
                  isActive: _currentBeatType == "MAJOR" && _isRunning,
                ),
                _buildCircle(
                  color: Colors.blue,
                  label: "MEDIUM", 
                  controller: _circle2Controller,
                  isActive: _currentBeatType == "MEDIUM" && _isRunning,
                ),
                _buildCircle(
                  color: Colors.cyan,
                  label: "MINOR",
                  controller: _circle3Controller,
                  isActive: _currentBeatType == "MINOR" && _isRunning,
                ),
              ],
            ),
            
            const SizedBox(height: 32),
            
            // Current beat information
            if (_isRunning) ...[
              Card(
                child: Padding(
                  padding: const EdgeInsets.all(16.0),
                  child: Column(
                    children: [
                      Text(
                        'Beat #${_currentBeat + 1} | Bar $_currentBar | $_currentTimeSignature',
                        style: Theme.of(context).textTheme.titleLarge,
                      ),
                      const SizedBox(height: 8),
                      Text(
                        'Beat Type: $_currentBeatType',
                        style: Theme.of(context).textTheme.titleMedium,
                      ),
                      const SizedBox(height: 8),
                      Text(
                        'Current Tempo: ${_currentTempo.toInt()} BPM',
                        style: Theme.of(context).textTheme.bodyLarge,
                      ),
                    ],
                  ),
                ),
              ),
            ],
            
            const SizedBox(height: 32),
            
            // Control buttons
            Row(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                ElevatedButton.icon(
                  onPressed: _isRunning ? null : _startTimer,
                  icon: const Icon(Icons.play_arrow),
                  label: const Text('Start Timer'),
                ),
                const SizedBox(width: 16),
                ElevatedButton.icon(
                  onPressed: _isRunning ? _stopTimer : null,
                  icon: const Icon(Icons.stop),
                  label: const Text('Stop Timer'),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildCircle({
    required Color color,
    required String label,
    required AnimationController controller,
    required bool isActive,
  }) {
    return Column(
      children: [
        AnimatedBuilder(
          animation: controller,
          builder: (context, child) {
            double scale = 1.0 + (controller.value * 0.5);
            double opacity = isActive ? 1.0 : 0.3;
            
            return Transform.scale(
              scale: scale,
              child: Container(
                width: 80,
                height: 80,
                decoration: BoxDecoration(
                  shape: BoxShape.circle,
                  color: color.withOpacity(opacity),
                  border: Border.all(
                    color: isActive ? Colors.white : Colors.grey,
                    width: 3,
                  ),
                  boxShadow: isActive ? [
                    BoxShadow(
                      color: color.withOpacity(0.6),
                      blurRadius: 15,
                      spreadRadius: 5,
                    )
                  ] : null,
                ),
              ),
            );
          },
        ),
        const SizedBox(height: 8),
        Text(
          label,
          style: TextStyle(
            fontWeight: FontWeight.bold,
            color: isActive ? Colors.black : Colors.grey,
          ),
        ),
      ],
    );
  }
}
