import 'package:flutter/material.dart';

class WizardFeatureSelector extends StatelessWidget {
  const WizardFeatureSelector({super.key});

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: .only(left: 16, right: 16, top: 16),
      child: Column(
        crossAxisAlignment: .start,
        children: [
          Text(
            "Customize your Instance",
            style: Theme.of(context).textTheme.titleLarge,
          ),
          // TODO: Change if no template was chosen.
          Text("Pick and choose features on your modules."),
        ],
      ),
    );
  }
}
