import 'package:flutter/material.dart';

class WizardInstanceDiscovery extends StatelessWidget {
  const WizardInstanceDiscovery({super.key});

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: .only(left: 16, right: 16, top: 16),
      child: Column(
        crossAxisAlignment: .start,
        children: [
          Text(
            "Searching for Instances",
            style: Theme.of(context).textTheme.titleLarge,
          ),
          // Customize if we're just connecting
          Text("Power on your instance, and we'll try and find it!"),
        ],
      ),
    );
  }
}
