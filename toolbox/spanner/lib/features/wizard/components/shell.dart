import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import 'package:spanner/components/window_controls.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:spanner/features/wizard/screens/from_scratch.dart';
import 'package:spanner/features/wizard/screens/template_selection.dart';
import 'package:spanner/features/wizard/screens/feature_selector.dart';
import 'package:spanner/features/wizard/screens/part_confirmation.dart';
import 'package:spanner/features/wizard/screens/part_assembly.dart';
import 'package:spanner/features/wizard/screens/instance_connection.dart';
import 'package:spanner/features/wizard/screens/instance_discovery.dart';
import 'package:spanner/features/wizard/screens/part_adoption.dart';
import 'package:spanner/features/wizard/screens/part_flashing.dart';
import 'package:spanner/features/wizard/screens/firmware_overview.dart';

enum WizardStep {
  welcome,
  templateSelection,
  featureSelector,
  partConfirmation,
  partAssembly,
  instanceConnection,
  firmwareOverview,
  partFlashing,
  instanceDiscovery,
  partAdoption,
}

class WizardState {
  final int currentStepIndex;
  final bool isStepValid;
  final bool isJustConnecting;
  final bool goingForward;

  const WizardState({
    required this.currentStepIndex,
    this.isStepValid = false,
    required this.isJustConnecting,
    this.goingForward = true,
  });

  WizardState copyWith({
    int? currentStepIndex,
    bool? isStepValid,
    bool? isJustConnecting,
    bool? goingForward,
  }) {
    return WizardState(
      currentStepIndex: currentStepIndex ?? this.currentStepIndex,
      isStepValid: isStepValid ?? this.isStepValid,
      isJustConnecting: isJustConnecting ?? this.isJustConnecting,
      goingForward: goingForward ?? this.goingForward,
    );
  }
}

class WizardNotifier extends Notifier<WizardState> {
  late final PageController pageController;
  bool _isInitialized = false;

  @override
  WizardState build() {
    pageController = PageController(initialPage: 0);

    ref.onDispose(() {
      pageController.dispose();
    });

    return const WizardState(currentStepIndex: 0, isJustConnecting: false);
  }

  void initialize(bool isJustConnecting) {
    if (_isInitialized) return;
    _isInitialized = true;

    final sequence = _getSequence(isJustConnecting);
    int initialPageIndex = 0;

    if (isJustConnecting) {
      initialPageIndex = sequence.indexOf(WizardStep.instanceDiscovery);
      if (initialPageIndex == -1) initialPageIndex = 0;
    }

    if (initialPageIndex != 0) {
      pageController.jumpToPage(initialPageIndex);
    }

    state = WizardState(
      currentStepIndex: initialPageIndex,
      isJustConnecting: isJustConnecting,
      isStepValid: !isJustConnecting,
    );
  }

  List<WizardStep> _getSequence(bool isJustConnecting) {
    if (isJustConnecting) {
      return [WizardStep.instanceDiscovery, WizardStep.instanceConnection];
    }
    return WizardStep.values;
  }

  List<WizardStep> get activeSequence {
    return _getSequence(state.isJustConnecting);
  }

  WizardStep get currentStep => activeSequence[state.currentStepIndex];
  double get progress => (state.currentStepIndex + 1) / activeSequence.length;
  bool get isFirstStep => state.currentStepIndex == 0;
  bool get isLastStep => state.currentStepIndex == activeSequence.length - 1;

  void setStepValid(bool isValid) {
    state = state.copyWith(isStepValid: isValid);
  }

  void setJustConnecting(bool isJustConnecting) {
    state = state.copyWith(isJustConnecting: isJustConnecting);
  }

  void nextStep() {
    if (isLastStep) return;

    state = state.copyWith(
      currentStepIndex: state.currentStepIndex + 1,
      isStepValid: false,
    );

    pageController.animateToPage(
      state.currentStepIndex,
      duration: const Duration(milliseconds: 300),
      curve: Curves.fastOutSlowIn,
    );
  }

  void previousStep() {
    if (isFirstStep) return;

    state = state.copyWith(
      currentStepIndex: state.currentStepIndex - 1,
      isStepValid: true,
    );

    pageController.animateToPage(
      state.currentStepIndex,
      duration: const Duration(milliseconds: 400),
      curve: Curves.fastOutSlowIn,
    );
  }
}

final wizardProvider = NotifierProvider.autoDispose(() => WizardNotifier());

class WizardShell extends ConsumerWidget {
  final GlobalKey<ScaffoldState> _scaffoldKey = GlobalKey<ScaffoldState>();

  WizardShell({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final wizardState = ref.watch(wizardProvider);
    final wizardNotifier = ref.read(wizardProvider.notifier);
    final isStepValid = wizardState.isStepValid;
    final state = GoRouterState.of(context);
    final isJustConnecting = state.uri.queryParameters['justConnect'] == 'true';

    WidgetsBinding.instance.addPostFrameCallback((_) {
      wizardNotifier.initialize(isJustConnecting);
    });

    return PopScope(
      canPop: !wizardNotifier.isFirstStep,
      onPopInvokedWithResult: (didPop, result) {
        if (didPop) return;
        wizardNotifier.previousStep();
      },
      child: Scaffold(
        key: _scaffoldKey,
        appBar: AppBar(
          leading: BackButton(onPressed: () => context.go("/")),
          title: Text("Setup Wizard"),
          actions: [WindowControls()],
          bottom: PreferredSize(
            preferredSize: const Size.fromHeight(4.0),
            child: TweenAnimationBuilder<double>(
              duration: const Duration(milliseconds: 400),
              curve: Curves
                  .fastOutSlowIn, // MD3 standard curve for progress transformations.
              tween: Tween<double>(
                begin: 0.0,
                end: _calculateProgress(ref, isJustConnecting),
              ),
              builder: (context, value, child) {
                return LinearProgressIndicator(
                  value: value,
                  borderRadius: BorderRadius.circular(4.0),
                  backgroundColor: Theme.of(
                    context,
                  ).colorScheme.surfaceContainerHighest.withValues(alpha: 0.3),
                  valueColor: AlwaysStoppedAnimation<Color>(
                    Theme.of(context).colorScheme.primary,
                  ),
                );
              },
            ),
          ),
        ),
        body: SizedBox.expand(
          child: PageView.builder(
            controller: wizardNotifier.pageController,
            physics: const NeverScrollableScrollPhysics(),
            itemCount: wizardNotifier.activeSequence.length,
            itemBuilder: (context, index) {
              final step = wizardNotifier.activeSequence[index];
              return _buildScreenForStep(step);
            },
          ),
        ),
        bottomNavigationBar: BottomAppBar(
          child: Row(
            children: [
              IconButton(
                icon: Icon(Icons.question_mark),
                onPressed: () => _showDocs(context, ""),
                tooltip: "Show help from the manual for this step.",
              ),
              Spacer(),
              IconButton(
                icon: Icon(Icons.arrow_back),
                onPressed: wizardNotifier.isFirstStep
                    ? null
                    : () => wizardNotifier.previousStep(),
                tooltip: "Previous step.",
              ),
              SizedBox(width: 8),
              FloatingActionButton.extended(
                // When disabled, FABs don't have a built-in 'null' style, hence manual styling.
                onPressed: isStepValid ? () => _handleNext(ref, context) : null,
                elevation: isStepValid ? null : 0,
                focusElevation: isStepValid ? null : 0,
                hoverElevation: isStepValid ? null : 0,
                backgroundColor: isStepValid
                    ? Theme.of(context).colorScheme.primaryContainer
                    : Theme.of(
                        context,
                      ).colorScheme.onSurface.withValues(alpha: 0.12),
                foregroundColor: isStepValid
                    ? Theme.of(context).colorScheme.onPrimaryContainer
                    : Theme.of(
                        context,
                      ).colorScheme.onSurface.withValues(alpha: 0.12),
                icon: const Icon(Icons.arrow_forward),
                label: Text(wizardNotifier.isLastStep ? "Finish" : "Next"),
              ),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildScreenForStep(WizardStep step) {
    return switch (step) {
      WizardStep.welcome => const WizardFromScratch(),
      WizardStep.templateSelection => const WizardTemplateSelection(),
      WizardStep.featureSelector => const WizardFeatureSelector(),
      WizardStep.partConfirmation => const WizardPartConfirmation(),
      WizardStep.partAssembly => const WizardPartAssembly(),
      WizardStep.firmwareOverview => const WizardFirmwareOverview(),
      WizardStep.partFlashing => const WizardPartFlashing(),
      WizardStep.instanceDiscovery => const WizardInstanceDiscovery(),
      WizardStep.instanceConnection => const WizardInstanceConnection(),
      WizardStep.partAdoption => const WizardPartAdoption(),
    };
  }

  // TODO: Make dependant on the number of modules and firmwares to flash.
  double _calculateProgress(WidgetRef ref, bool isJustConnecting) {
    final notifier = ref.read(wizardProvider.notifier);
    final activeSequence = notifier.activeSequence;

    final int currentIndex = activeSequence.indexOf(notifier.currentStep);

    if (currentIndex == -1) return 0.0;

    if (isJustConnecting) {
      if (notifier.currentStep == WizardStep.instanceDiscovery) return 0.33;
      if (notifier.currentStep == WizardStep.instanceConnection) return 0.66;
      return 1.0;
    } else {
      return (currentIndex + 1) / activeSequence.length;
    }
  }

  void _handleNext(WidgetRef ref, BuildContext context) {
    if (ref.read(wizardProvider.notifier).isLastStep) {
      context.go('/configurator/test');
    } else {
      ref.read(wizardProvider.notifier).nextStep();
    }
  }

  void _showDocs(BuildContext context, String path) {
    _scaffoldKey.currentState?.showBottomSheet((context) {
      return Container(
        height: MediaQuery.of(context).size.height * 0.8,
        width: double.infinity,
        padding: const EdgeInsets.all(24.0),
        decoration: BoxDecoration(
          color: Theme.of(context).colorScheme.surfaceContainer,
          borderRadius: const BorderRadius.vertical(top: Radius.circular(28.0)),
        ),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              "Step Documentation",
              style: Theme.of(context).textTheme.titleLarge,
            ),
            const SizedBox(height: 16),
            Expanded(
              child: Center(
                child: Text("Future WebView loading docs for:\n$path"),
              ),
            ),
          ],
        ),
      );
    }, elevation: 8);
  }
}
