import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';

class LandingPage extends StatelessWidget {
  const LandingPage({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: Text("Spanner")),
      body: Column(
        children: [
          Text("Welcome to Spanner!"),
          TextButton(
            child: Text("Connect to an Existing Instance"),
            onPressed: () {
              context.go("/configurator");
            },
          ),
          // TODO: Fix Dividers
          Row(children: [Divider(), Text("OR"), Divider()]),
          TextButton(
            child: Text("Try the config wizard!"),
            onPressed: () {
              context.go("/wizard");
            },
          ),
        ],
      ),
    );
  }
}
