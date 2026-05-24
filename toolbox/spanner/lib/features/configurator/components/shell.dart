import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';

class NavDestinationConfig {
  final String id; // Unique identifier for reordering/keys
  final String label; // Text shown below the icon
  final String routePath; // The router path (base) target
  final IconData icon; // Default outlined icon
  final IconData selectedIcon; // Filled icon when active

  const NavDestinationConfig({
    required this.id,
    required this.label,
    required this.routePath,
    required this.icon,
    required this.selectedIcon,
  });
}

// Your default layout ordering blueprint
const List<NavDestinationConfig> defaultNavDestinations = [
  NavDestinationConfig(
    id: 'overview',
    label: 'Overview',
    routePath: '/configurator',
    icon: Icons.settings_outlined,
    selectedIcon: Icons.settings,
  ),
  NavDestinationConfig(
    id: 'modules',
    label: 'Modules',
    routePath: '/configurator/modules',
    icon: Icons.extension_outlined,
    selectedIcon: Icons.extension,
  ),
  NavDestinationConfig(
    id: 'gestures',
    label: 'Gestures',
    routePath: '/configurator/gestures',
    icon: Icons.gesture_outlined,
    selectedIcon: Icons.gesture,
  ),
  NavDestinationConfig(
    id: 'apps',
    label: 'Apps',
    routePath: '/configurator/apps',
    icon: Icons.code_outlined,
    selectedIcon: Icons.code,
  ),
  NavDestinationConfig(
    id: 'repos',
    label: 'Repos',
    routePath: '/configurator/repos',
    icon: Icons.cloud_outlined,
    selectedIcon: Icons.cloud,
  ),
];

class ConfiguratorShell extends StatelessWidget {
  final StatefulNavigationShell navigationShell;
  // TODO: Store the order of navbar destinations in settings
  final List<NavDestinationConfig> activeDestinations = defaultNavDestinations;

  const ConfiguratorShell({super.key, required this.navigationShell});

  void _onTabSelected(int index) {
    navigationShell.goBranch(
      index,
      initialLocation: index == navigationShell.currentIndex,
    );
  }

  @override
  Widget build(BuildContext context) {
    final int selectedIndex = navigationShell.currentIndex;

    return LayoutBuilder(
      builder: (context, constraints) {
        final bool isSmall = constraints.maxWidth < 600;

        return Scaffold(
          appBar: AppBar(
            title: Text("Spanner"),
            actions: [
              Container(
                padding: EdgeInsetsGeometry.only(right: 6.0),
                child: IconButton(
                  icon: Icon(Icons.settings_outlined),
                  onPressed: () {
                    context.push("/settings");
                  },
                ),
              ),
            ],
          ),
          bottomNavigationBar: isSmall
              ? NavigationBar(
                  selectedIndex: selectedIndex,
                  onDestinationSelected: _onTabSelected,
                  destinations: activeDestinations.map((dest) {
                    return NavigationDestination(
                      icon: Icon(dest.icon),
                      selectedIcon: Icon(dest.selectedIcon),
                      label: dest.label,
                    );
                  }).toList(),
                )
              : null,

          body: Row(
            children: [
              if (!isSmall) ...[
                NavigationRail(
                  selectedIndex: selectedIndex,
                  onDestinationSelected: _onTabSelected,
                  labelType: NavigationRailLabelType.all,
                  destinations: activeDestinations.map((dest) {
                    return NavigationRailDestination(
                      icon: Icon(dest.icon),
                      selectedIcon: Icon(dest.selectedIcon),
                      label: Text(dest.label),
                    );
                  }).toList(),
                ),
              ],

              Expanded(child: navigationShell),
            ],
          ),
        );
      },
    );
  }
}
