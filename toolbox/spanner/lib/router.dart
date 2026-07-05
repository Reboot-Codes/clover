import 'package:go_router/go_router.dart';

// Landings
import 'package:spanner/features/welcome/screens/landing.dart';

// Global Settings
import 'package:spanner/features/settings/components/shell.dart';
import 'package:spanner/features/settings/screens/categories.dart';

// Configurator
import 'package:spanner/features/configurator/components/shell.dart';
import 'package:spanner/features/configurator/screens/overview.dart';
import 'package:spanner/features/configurator/screens/modules/overview.dart';
import 'package:spanner/features/configurator/screens/gestures/quick_settings.dart';
import 'package:spanner/features/configurator/screens/apps/apps_list.dart';
import 'package:spanner/features/configurator/screens/repos/repo_list.dart';

// Repository Manager
import 'package:spanner/features/repo_manager/components/shell.dart';
import 'package:spanner/features/repo_manager/screens/overview.dart';
import 'package:spanner/features/repo_manager/screens/modules/list.dart';
import 'package:spanner/features/repo_manager/screens/gestures/list.dart';
import 'package:spanner/features/repo_manager/screens/apps/list.dart';

// Build Wizard
import 'package:spanner/features/wizard/components/shell.dart';

// GoRouter configuration
final router = GoRouter(
  initialLocation: '/',
  routes: <RouteBase>[
    // Configurator
    GoRoute(
      path: "/configurator/:instanceId",
      redirect: (context, state) {
        if (state.uri.path.endsWith(state.pathParameters["instanceId"]!)) {
          return "${state.uri.path}/overview";
        }

        return null;
      },
      routes: [
        StatefulShellRoute.indexedStack(
          builder: (context, state, navigationShell) {
            return ConfiguratorShell(navigationShell: navigationShell);
          },
          branches: [
            StatefulShellBranch(
              routes: [
                GoRoute(
                  path: '/overview',
                  builder: (context, state) => const ConfiguratorOverview(),
                ),
              ],
            ),
            StatefulShellBranch(
              routes: [
                GoRoute(
                  path: '/modules',
                  builder: (context, state) =>
                      const ConfiguratorModulesOverview(),
                ),
              ],
            ),
            StatefulShellBranch(
              routes: [
                GoRoute(
                  path: '/gestures',
                  builder: (context, state) => const GestureQuickSettings(),
                ),
              ],
            ),
            StatefulShellBranch(
              routes: [
                GoRoute(
                  path: '/apps',
                  builder: (context, state) => const ConfiguratorAppsList(),
                ),
              ],
            ),
            StatefulShellBranch(
              routes: [
                GoRoute(
                  path: '/repos',
                  builder: (context, state) => const RepoList(),
                ),
              ],
            ),
          ],
        ),
      ],
    ),

    // RepoManager and related detail routes
    GoRoute(
      path: "/repo_manager/:repoId",
      redirect: (context, state) {
        if (state.uri.path.endsWith(state.pathParameters["repoId"]!)) {
          return "${state.uri.path}/overview";
        }

        return null;
      },
      routes: [
        StatefulShellRoute.indexedStack(
          builder: (context, state, navigationShell) {
            return RepoManagerShell(navigationShell: navigationShell);
          },
          branches: [
            StatefulShellBranch(
              routes: [
                GoRoute(
                  path: '/overview',
                  builder: (context, state) => const RepoOverview(),
                ),
              ],
            ),
            StatefulShellBranch(
              routes: [
                GoRoute(
                  path: '/modules',
                  builder: (context, state) => const RepoModulesList(),
                ),
              ],
            ),
            StatefulShellBranch(
              routes: [
                GoRoute(
                  path: '/gestures',
                  builder: (context, state) => const RepoGestureList(),
                ),
              ],
            ),
            StatefulShellBranch(
              routes: [
                GoRoute(
                  path: '/apps',
                  builder: (context, state) => const RepoAppsList(),
                ),
              ],
            ),
          ],
        ),
      ],
    ),

    GoRoute(path: "/wizard", builder: (context, state) => WizardShell()),

    ShellRoute(
      builder: (context, state, child) {
        return SettingsShell(child: child);
      },
      routes: [
        GoRoute(
          path: "/settings",
          builder: (context, state) => const SettingsCategories(),
        ),
      ],
    ),
    GoRoute(path: "/", builder: (context, state) => const LandingPage()),
  ],
);
