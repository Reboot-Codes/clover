import 'package:flutter/material.dart';

class WizardFirmwareOverview extends StatelessWidget {
  const WizardFirmwareOverview({super.key});

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: .only(left: 16, right: 16, top: 16),
      child: Column(
        crossAxisAlignment: .start,
        children: [
          Text(
            "Customize your Firmware",
            style: Theme.of(context).textTheme.titleLarge,
          ),
          Text(
            "We've ensured that your firmware will work with the modules you've built, but you can add extra features here.",
          ),
        ],
      ),
    );
  }
}
