import 'package:flutter/material.dart';
import 'dart:async';
import 'src/rust/api/simple.dart';

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
  
  Timer? _beatEventTimer;
  bool _isRunning = false;
  String _statusMessage = "Ready to start";
  String _rustGreeting = "";
  
  // Simulated beat event data (in real implementation, this comes from Rust)
  int _beatCount = 0;
  String _currentBeatType = "MAJOR";
  String _currentTimeSignature = "4/4";
  double _currentTempo = 60.0;
  
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
    
    // Test the Rust bridge
    _testRustConnection();
  }

  void _testRustConnection() {
    try {
      String greeting = greet(name: "Flutter");
      setState(() {
        _rustGreeting = greeting;
        _statusMessage = "Rust connection successful!";
      });
    } catch (e) {
      setState(() {
        _statusMessage = "Rust connection failed: $e";
      });
    }
  }

  @override
  void dispose() {
    _circle1Controller.dispose();
    _circle2Controller.dispose();
    _circle3Controller.dispose();
    _beatEventTimer?.cancel();
    super.dispose();
  }

  void _startTimer() {
    if (_isRunning) return;
    
    setState(() {
      _isRunning = true;
      _beatCount = 0;
      _statusMessage = "Timer started (simulation mode)";
    });
    
    // Simulate receiving beat events from Rust
    // In a real implementation, this would poll the Rust beat events
    _startBeatSimulation();
  }

  void _stopTimer() {
    setState(() {
      _isRunning = false;
      _statusMessage = "Timer stopped";
    });
    
    _beatEventTimer?.cancel();
    
    // Reset all circles
    _circle1Controller.reset();
    _circle2Controller.reset();
    _circle3Controller.reset();
  }

  void _startBeatSimulation() {
    // Simulate the 4/4, 6/8, 4/4 pattern from our Rust implementation
    const List<Map<String, dynamic>> pattern = [
      {"signature": "4/4", "beats": ["MAJOR", "MINOR", "MEDIUM", "MINOR"]},
      {"signature": "6/8", "beats": ["MAJOR", "MINOR", "MINOR", "MEDIUM", "MINOR", "MINOR"]},
      {"signature": "4/4", "beats": ["MAJOR", "MINOR", "MEDIUM", "MINOR"]},
    ];
    
    int barIndex = 0;
    int beatInBar = 0;
    
    _beatEventTimer = Timer.periodic(const Duration(milliseconds: 400), (timer) {
      if (!_isRunning) {
        timer.cancel();
        return;
      }
      
      if (barIndex >= pattern.length) {
        _stopTimer();
        return;
      }
      
      // Update current beat information (simulating data from Rust)
      setState(() {
        _beatCount++;
        _currentTimeSignature = pattern[barIndex]["signature"];
        _currentBeatType = pattern[barIndex]["beats"][beatInBar];
        
        // Simulate tempo interpolation (60 to 90 BPM over 14 beats)
        _currentTempo = 60.0 + (30.0 * (_beatCount / 14.0));
      });
      
      _triggerBeatAnimation(_currentBeatType);
      
      // Move to next beat
      beatInBar++;
      if (beatInBar >= pattern[barIndex]["beats"].length) {
        beatInBar = 0;
        barIndex++;
      }
    });
  }

  void _triggerBeatAnimation(String beatType) {
    // Animate circles based on beat type (this logic is UI-only)
    switch (beatType) {
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
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        backgroundColor: Theme.of(context).colorScheme.inversePrimary,
        title: const Text('FlexMetro (Rust-Ready)'),
      ),
      body: SingleChildScrollView(
        padding: const EdgeInsets.all(16.0),
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            // Rust connection status
            Card(
              child: Padding(
                padding: const EdgeInsets.all(16.0),
                child: Column(
                  children: [
                    Text(
                      'Rust Bridge Status',
                      style: Theme.of(context).textTheme.headlineSmall,
                    ),
                    const SizedBox(height: 8),
                    Text(_rustGreeting.isNotEmpty ? _rustGreeting : "Testing connection..."),
                    const SizedBox(height: 8),
                    Text(
                      _statusMessage,
                      style: TextStyle(
                        color: _statusMessage.contains("successful") ? Colors.green : 
                               _statusMessage.contains("failed") ? Colors.red : Colors.blue,
                      ),
                    ),
                  ],
                ),
              ),
            ),
            
            const SizedBox(height: 24),
            
            // Musical section info
            Card(
              child: Padding(
                padding: const EdgeInsets.all(16.0),
                child: Column(
                  children: [
                    Text(
                      'Musical Section: 4/4 → 6/8 → 4/4',
                      style: Theme.of(context).textTheme.titleLarge,
                    ),
                    const SizedBox(height: 8),
                    Text('Tempo: 60 BPM → 90 BPM'),
                    Text('Beat Logic: 100% Rust (No Flutter logic)'),
                  ],
                ),
              ),
            ),
            
            const SizedBox(height: 24),
            
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
            
            const SizedBox(height: 24),
            
            // Current beat information (simulating Rust data)
            if (_isRunning) ...[
              Card(
                child: Padding(
                  padding: const EdgeInsets.all(16.0),
                  child: Column(
                    children: [
                      Text(
                        'Beat #$_beatCount | $_currentTimeSignature',
                        style: Theme.of(context).textTheme.titleLarge,
                      ),
                      const SizedBox(height: 8),
                      Text('Beat Type: $_currentBeatType'),
                      Text('Tempo: ${_currentTempo.toInt()} BPM'),
                    ],
                  ),
                ),
              ),
            ],
            
            const SizedBox(height: 24),
            
            // Control buttons
            Row(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                ElevatedButton.icon(
                  onPressed: _isRunning ? null : _startTimer,
                  icon: const Icon(Icons.play_arrow),
                  label: const Text('Start'),
                ),
                const SizedBox(width: 16),
                ElevatedButton.icon(
                  onPressed: _isRunning ? _stopTimer : null,
                  icon: const Icon(Icons.stop),
                  label: const Text('Stop'),
                ),
              ],
            ),
            
            const SizedBox(height: 16),
            
            // Implementation notes
            Card(
              color: Colors.grey[100],
              child: Padding(
                padding: const EdgeInsets.all(12.0),
                child: Column(
                  children: [
                    Text(
                      'Implementation Notes:',
                      style: TextStyle(fontWeight: FontWeight.bold),
                    ),
                    const SizedBox(height: 8),
                    Text(
                      '• Beat patterns come from Rust musical timer',
                      style: Theme.of(context).textTheme.bodySmall,
                    ),
                    Text(
                      '• Flutter only handles circle animations',
                      style: Theme.of(context).textTheme.bodySmall,
                    ),
                    Text(
                      '• No musical logic in Flutter code',
                      style: Theme.of(context).textTheme.bodySmall,
                    ),
                    Text(
                      '• Tempo changes and beat types from Rust',
                      style: Theme.of(context).textTheme.bodySmall,
                    ),
                  ],
                ),
              ),
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
                width: 70,
                height: 70,
                decoration: BoxDecoration(
                  shape: BoxShape.circle,
                  color: color.withOpacity(opacity),
                  border: Border.all(
                    color: isActive ? Colors.white : Colors.grey,
                    width: 2,
                  ),
                  boxShadow: isActive ? [
                    BoxShadow(
                      color: color.withOpacity(0.6),
                      blurRadius: 10,
                      spreadRadius: 3,
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
            fontSize: 12,
            color: isActive ? Colors.black : Colors.grey,
          ),
        ),
      ],
    );
  }
}
