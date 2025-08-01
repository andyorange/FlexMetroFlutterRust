import 'package:flutter/material.dart';
import 'dart:async';
import 'src/rust/api/flutter_interface.dart';

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
  
  // Current beat information - all comes from Rust
  FlutterBeatEvent? _currentBeatEvent;
  String _statusMessage = "Ready to start";
  
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
    
    // Initialize the Rust timer
    _initializeRustTimer();
  }

  @override
  void dispose() {
    _circle1Controller.dispose();
    _circle2Controller.dispose();
    _circle3Controller.dispose();
    _beatEventTimer?.cancel();
    super.dispose();
  }

  void _initializeRustTimer() {
    initFlutterTimer();
    setState(() {
      _statusMessage = "Timer initialized";
    });
  }

  void _startTimer() {
    if (_isRunning) return;
    
    // Define our musical section: 4/4, 6/8, 4/4 (same as Rust test)
    List<List<int>> bars = [
      [4, 4], // 4/4
      [6, 8], // 6/8 
      [4, 4], // 4/4
    ];
    
    // Start the Rust timer with the configuration
    String result = startFlutterMusicalTimerWithConfig(
      bars: bars.map((bar) => (bar[0], bar[1])).toList(),
      startTempoBpm: 60.0, 
      endTempoBpm: 90.0,
    );
    
    setState(() {
      _isRunning = true;
      _statusMessage = result;
    });
    
    // Start polling for beat events from Rust
    _startBeatEventPolling();
  }

  void _stopTimer() {
    String result = stopFlutterMusicalTimer();
    
    setState(() {
      _isRunning = false;
      _currentBeatEvent = null;
      _statusMessage = result;
    });
    
    _beatEventTimer?.cancel();
    
    // Reset all circles
    _circle1Controller.reset();
    _circle2Controller.reset();
    _circle3Controller.reset();
  }

  void _startBeatEventPolling() {
    // Poll for beat events from Rust every 10ms
    _beatEventTimer = Timer.periodic(const Duration(milliseconds: 10), (timer) {
      if (!_isRunning) {
        timer.cancel();
        return;
      }
      
      List<FlutterBeatEvent> events = getFlutterBeatEvents();
      if (events.isNotEmpty) {
        // Process the latest beat event
        FlutterBeatEvent latestEvent = events.last;
        
        // Only update if this is a new beat
        if (_currentBeatEvent == null || 
            latestEvent.beatNumber != _currentBeatEvent!.beatNumber ||
            latestEvent.elapsedMs != _currentBeatEvent!.elapsedMs) {
          
          setState(() {
            _currentBeatEvent = latestEvent;
          });
          
          _triggerBeatAnimation(latestEvent.beatType);
        }
      }
    });
  }

  void _triggerBeatAnimation(String beatType) {
    // Animate circles based on beat type from Rust
    switch (beatType.toLowerCase()) {
      case "major":
        _circle1Controller.forward().then((_) => _circle1Controller.reverse());
        break;
      case "medium":
        _circle2Controller.forward().then((_) => _circle2Controller.reverse());
        break;
      case "minor":
        _circle3Controller.forward().then((_) => _circle3Controller.reverse());
        break;
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        backgroundColor: Theme.of(context).colorScheme.inversePrimary,
        title: const Text('FlexMetro Musical Timer (Rust-Powered)'),
      ),
      body: SingleChildScrollView(
        child: Padding(
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
                      'Tempo: 60 BPM → 90 BPM (from Rust)',
                      style: Theme.of(context).textTheme.titleMedium,
                    ),
                    const SizedBox(height: 8),
                    Text(
                      'Status: $_statusMessage',
                      style: Theme.of(context).textTheme.bodyMedium,
                    ),
                  ],
                ),
              ),
            ),
            
            const SizedBox(height: 32),
            
            // Three circles display - animations triggered by Rust beat events
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceEvenly,
              children: [
                _buildCircle(
                  color: Colors.yellow,
                  label: "MAJOR",
                  controller: _circle1Controller,
                  isActive: _currentBeatEvent?.beatType.toLowerCase() == "major" && _isRunning,
                ),
                _buildCircle(
                  color: Colors.blue,
                  label: "MEDIUM", 
                  controller: _circle2Controller,
                  isActive: _currentBeatEvent?.beatType.toLowerCase() == "medium" && _isRunning,
                ),
                _buildCircle(
                  color: Colors.cyan,
                  label: "MINOR",
                  controller: _circle3Controller,
                  isActive: _currentBeatEvent?.beatType.toLowerCase() == "minor" && _isRunning,
                ),
              ],
            ),
            
            const SizedBox(height: 32),
            
            // Current beat information - all from Rust
            if (_isRunning && _currentBeatEvent != null) ...[
              Card(
                child: Padding(
                  padding: const EdgeInsets.all(16.0),
                  child: Column(
                    children: [
                      Text(
                        'Beat #${_currentBeatEvent!.beatNumber} | ${_currentBeatEvent!.timeSignature}',
                        style: Theme.of(context).textTheme.titleLarge,
                      ),
                      const SizedBox(height: 8),
                      Text(
                        'Beat Type: ${_currentBeatEvent!.beatType}',
                        style: Theme.of(context).textTheme.titleMedium,
                      ),
                      const SizedBox(height: 8),
                      Text(
                        'Current Tempo: ${_currentBeatEvent!.tempoBpm.toInt()} BPM',
                        style: Theme.of(context).textTheme.bodyLarge,
                      ),
                      const SizedBox(height: 8),
                      Text(
                        'Elapsed: ${_currentBeatEvent!.elapsedMs}ms | Interval: ${_currentBeatEvent!.intervalMs}ms',
                        style: Theme.of(context).textTheme.bodySmall,
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
                  label: const Text('Start Rust Timer'),
                ),
                const SizedBox(width: 16),
                ElevatedButton.icon(
                  onPressed: _isRunning ? _stopTimer : null,
                  icon: const Icon(Icons.stop),
                  label: const Text('Stop Timer'),
                ),
              ],
            ),
            
            const SizedBox(height: 16),
            
            // Note about the implementation
            Card(
              color: Colors.grey[100],
              child: Padding(
                padding: const EdgeInsets.all(12.0),
                child: Text(
                  'Note: All beat logic is now handled by Rust. Flutter only displays the results.',
                  style: Theme.of(context).textTheme.bodySmall,
                  textAlign: TextAlign.center,
                ),
              ),
            ),
          ],
        ),
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
