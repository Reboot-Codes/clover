import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import 'package:spanner/components/window_controls.dart';

class WelcomeAction {
  final IconData icon;
  final String label;
  final String description;
  final String path;

  const WelcomeAction({
    required this.icon,
    required this.label,
    required this.description,
    required this.path,
  });
}

const List<WelcomeAction> defaultWelcomeActions = [
  WelcomeAction(
    icon: Icons.add_box,
    label: "Build",
    description: "Build a new Clover instance.",
    path: "/wizard",
  ),
  WelcomeAction(
    icon: Icons.cloud,
    label: "Repo Management",
    description: "View and edit known repositories.",
    path: "/repo_manager",
  ),
  WelcomeAction(
    icon: Icons.book,
    label: "Docs",
    description: "Read the Clover Manual.",
    path: "/docs",
  ),
];

class SubtleCardButton extends StatelessWidget {
  final Widget child;
  final void Function()? onTap;

  const SubtleCardButton({super.key, required this.child, this.onTap});

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return Padding(
      padding: const EdgeInsets.only(bottom: 8.0),
      child: Material(
        color: theme.colorScheme.surfaceContainerLow,
        borderRadius: BorderRadius.circular(8),
        child: onTap != null
            ? InkWell(
                borderRadius: BorderRadius.circular(8),
                onTap: onTap,
                child: Padding(
                  padding: const EdgeInsets.all(12.0),
                  child: child,
                ),
              )
            : Padding(padding: const EdgeInsets.all(12.0), child: child),
      ),
    );
  }
}

class LandingPage extends StatelessWidget {
  const LandingPage({super.key});

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final bodyFontSize = theme.textTheme.bodyMedium?.fontSize ?? 14.0;
    final iconSize = (theme.iconTheme.size ?? 24.0) * 1.5;

    // TODO: Replace with instance list
    final List<String> mockInstances = ["f1tzs-fursuit", "f1tzs-dailywear"];

    return Scaffold(
      appBar: AppBar(
        title: const Text("Spanner"),
        actions: [
          Container(
            padding: EdgeInsetsGeometry.only(right: 6.0),
            child: IconButton(
              icon: Icon(Icons.settings_outlined),
              tooltip: "Settings",
              onPressed: () {
                context.push("/settings");
              },
            ),
          ),
          WindowControls(),
        ],
      ),
      body: Padding(
        padding: const EdgeInsets.all(16.0),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              "Welcome to Spanner, the official configuration tool for Clover.",
            ),
            const SizedBox(height: 24),
            Expanded(
              child: LayoutBuilder(
                builder: (context, constraints) {
                  final bool isSmall = constraints.maxWidth < 600;

                  final Widget connectionsCard = SizedBox(
                    width: double.infinity,
                    child: Card(
                      clipBehavior: .hardEdge,
                      child: Padding(
                        padding: const EdgeInsets.all(16.0),
                        child: Column(
                          crossAxisAlignment: .start,
                          mainAxisSize: .min,
                          children: [
                            SubtleCardButton(
                              child: Row(
                                children: [
                                  Expanded(
                                    child: Column(
                                      mainAxisSize: .min,
                                      crossAxisAlignment: .start,
                                      children: [
                                        Row(
                                          children: [
                                            Icon(Icons.star, size: iconSize),
                                            const SizedBox(width: 12),
                                            Expanded(
                                              child: Column(
                                                mainAxisSize: .min,
                                                crossAxisAlignment: .start,
                                                children: [
                                                  Text(
                                                    "Previously Connected",
                                                    style: TextStyle(
                                                      fontWeight: .bold,
                                                      fontSize:
                                                          bodyFontSize * 1.25,
                                                    ),
                                                  ),
                                                  Text(
                                                    "Previously paired instances.",
                                                    style: TextStyle(
                                                      color: Colors.grey,
                                                    ),
                                                  ),
                                                ],
                                              ),
                                            ),
                                          ],
                                        ),
                                      ],
                                    ),
                                  ),
                                  const SizedBox(height: 8),
                                ],
                              ),
                            ),

                            const SizedBox(height: 8),

                            if (mockInstances.isEmpty)
                              const Padding(
                                padding: .symmetric(vertical: 16.0),
                                child: Text(
                                  "No instances detected; build or connect to an existing one for it to show up here!",
                                ),
                              )
                            else
                              ...mockInstances.map(
                                (instanceName) => SubtleCardButton(
                                  child: Row(
                                    children: [
                                      const Icon(Icons.devices, size: 20),
                                      const SizedBox(width: 12),
                                      Expanded(
                                        child: Text(
                                          instanceName,
                                          style: const TextStyle(
                                            fontWeight: .w500,
                                          ),
                                        ),
                                      ),
                                      const Icon(
                                        Icons.arrow_forward_ios,
                                        size: 14,
                                      ),
                                    ],
                                  ),
                                  onTap: () => context.push(
                                    "/configurator/$instanceName",
                                  ),
                                ),
                              ),

                            const Divider(),

                            SubtleCardButton(
                              child: Row(
                                children: [
                                  Expanded(
                                    child: Column(
                                      mainAxisSize: .min,
                                      crossAxisAlignment: .start,
                                      children: [
                                        Row(
                                          children: [
                                            Icon(Icons.link, size: iconSize),
                                            const SizedBox(width: 12),
                                            Expanded(
                                              child: Column(
                                                mainAxisSize: .min,
                                                crossAxisAlignment: .start,
                                                children: [
                                                  Text(
                                                    "Connect",
                                                    style: TextStyle(
                                                      fontWeight: .bold,
                                                      fontSize:
                                                          bodyFontSize * 1.25,
                                                    ),
                                                  ),
                                                  Text(
                                                    "Search for a new instance to connect to.",
                                                    style: TextStyle(
                                                      color: Colors.grey,
                                                    ),
                                                  ),
                                                ],
                                              ),
                                            ),
                                          ],
                                        ),
                                      ],
                                    ),
                                  ),
                                  const Icon(Icons.arrow_forward_ios, size: 14),
                                ],
                              ),
                              onTap: () =>
                                  context.push("/wizard?justConnect=true"),
                            ),
                          ],
                        ),
                      ),
                    ),
                  );

                  final Widget additionalActionsList = Column(
                    mainAxisSize: .min,
                    children: defaultWelcomeActions.map((action) {
                      return SizedBox(
                        width: double.infinity,
                        child: Card(
                          clipBehavior: .hardEdge,
                          margin: const EdgeInsets.only(bottom: 12.0),
                          child: InkWell(
                            onTap: () => context.push(action.path),
                            child: Padding(
                              padding: const EdgeInsets.all(16.0),
                              child: Row(
                                children: [
                                  Icon(
                                    action.icon,
                                    size: theme.iconTheme.size ?? 24.0,
                                  ),
                                  const SizedBox(width: 16),
                                  Expanded(
                                    child: Column(
                                      crossAxisAlignment: .start,
                                      mainAxisSize: .min,
                                      children: [
                                        Text(
                                          action.label,
                                          style: const TextStyle(
                                            fontWeight: .bold,
                                          ),
                                        ),
                                        const SizedBox(height: 4),
                                        Text(
                                          action.description,
                                          style: TextStyle(color: Colors.grey),
                                        ),
                                      ],
                                    ),
                                  ),
                                  const Icon(Icons.chevron_right),
                                ],
                              ),
                            ),
                          ),
                        ),
                      );
                    }).toList(),
                  );

                  return SingleChildScrollView(
                    physics: isSmall
                        ? const ClampingScrollPhysics()
                        : const NeverScrollableScrollPhysics(),
                    child: Flex(
                      direction: isSmall ? .vertical : .horizontal,
                      crossAxisAlignment: .start,
                      children: [
                        isSmall
                            ? connectionsCard
                            : Expanded(child: connectionsCard),
                        const SizedBox(width: 24, height: 24),
                        isSmall
                            ? additionalActionsList
                            : Expanded(child: additionalActionsList),
                      ],
                    ),
                  );
                },
              ),
            ),
          ],
        ),
      ),
    );
  }
}
