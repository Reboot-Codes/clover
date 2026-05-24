import 'dart:io';
import 'package:flutter/material.dart';

class WindowControls extends StatelessWidget {
  const WindowControls({super.key});

  bool isDesktop() {
    // TODO: Add OHOS support.
    if (Platform.isLinux || Platform.isWindows || Platform.isMacOS) {
      return true;
    } else {
      return false;
    }
  }

  @override
  Widget build(BuildContext context) {
    return Row(
      children: isDesktop()
          ? [
              VerticalDivider(width: 16, indent: 16, endIndent: 16),
              IconButton(
                icon: Icon(Icons.minimize),
                tooltip: "Minimize",
                onPressed: () {},
              ),
              IconButton(
                icon: Icon(Icons.square_outlined),
                tooltip: "Maximize",
                onPressed: () {},
              ),
              Container(
                padding: EdgeInsetsGeometry.only(right: 6.0),
                child: IconButton(
                  icon: Icon(Icons.close),
                  tooltip: "Close",
                  onPressed: () {},
                ),
              ),
            ]
          : [],
    );
  }
}
