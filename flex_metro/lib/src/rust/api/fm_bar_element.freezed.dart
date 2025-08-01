// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'fm_bar_element.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;

/// @nodoc
mixin _$BarBeatData {
  TypedData get field0;

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is BarBeatData &&
            (identical(other.field0, field0) || other.field0 == field0));
  }

  @override
  int get hashCode => Object.hash(runtimeType, field0);

  @override
  String toString() {
    return 'BarBeatData(field0: $field0)';
  }
}

/// @nodoc
class $BarBeatDataCopyWith<$Res> {
  $BarBeatDataCopyWith(BarBeatData _, $Res Function(BarBeatData) __);
}

/// Adds pattern-matching-related methods to [BarBeatData].
extension BarBeatDataPatterns on BarBeatData {
  /// A variant of `map` that fallback to returning `orElse`.
  ///
  /// It is equivalent to doing:
  /// ```dart
  /// switch (sealedClass) {
  ///   case final Subclass value:
  ///     return ...;
  ///   case _:
  ///     return orElse();
  /// }
  /// ```

  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(BarBeatData_Int value)? int,
    TResult Function(BarBeatData_Float value)? float,
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case BarBeatData_Int() when int != null:
        return int(_that);
      case BarBeatData_Float() when float != null:
        return float(_that);
      case _:
        return orElse();
    }
  }

  /// A `switch`-like method, using callbacks.
  ///
  /// Callbacks receives the raw object, upcasted.
  /// It is equivalent to doing:
  /// ```dart
  /// switch (sealedClass) {
  ///   case final Subclass value:
  ///     return ...;
  ///   case final Subclass2 value:
  ///     return ...;
  /// }
  /// ```

  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(BarBeatData_Int value) int,
    required TResult Function(BarBeatData_Float value) float,
  }) {
    final _that = this;
    switch (_that) {
      case BarBeatData_Int():
        return int(_that);
      case BarBeatData_Float():
        return float(_that);
    }
  }

  /// A variant of `map` that fallback to returning `null`.
  ///
  /// It is equivalent to doing:
  /// ```dart
  /// switch (sealedClass) {
  ///   case final Subclass value:
  ///     return ...;
  ///   case _:
  ///     return null;
  /// }
  /// ```

  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(BarBeatData_Int value)? int,
    TResult? Function(BarBeatData_Float value)? float,
  }) {
    final _that = this;
    switch (_that) {
      case BarBeatData_Int() when int != null:
        return int(_that);
      case BarBeatData_Float() when float != null:
        return float(_that);
      case _:
        return null;
    }
  }

  /// A variant of `when` that fallback to an `orElse` callback.
  ///
  /// It is equivalent to doing:
  /// ```dart
  /// switch (sealedClass) {
  ///   case Subclass(:final field):
  ///     return ...;
  ///   case _:
  ///     return orElse();
  /// }
  /// ```

  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(Int32List field0)? int,
    TResult Function(Float32List field0)? float,
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case BarBeatData_Int() when int != null:
        return int(_that.field0);
      case BarBeatData_Float() when float != null:
        return float(_that.field0);
      case _:
        return orElse();
    }
  }

  /// A `switch`-like method, using callbacks.
  ///
  /// As opposed to `map`, this offers destructuring.
  /// It is equivalent to doing:
  /// ```dart
  /// switch (sealedClass) {
  ///   case Subclass(:final field):
  ///     return ...;
  ///   case Subclass2(:final field2):
  ///     return ...;
  /// }
  /// ```

  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(Int32List field0) int,
    required TResult Function(Float32List field0) float,
  }) {
    final _that = this;
    switch (_that) {
      case BarBeatData_Int():
        return int(_that.field0);
      case BarBeatData_Float():
        return float(_that.field0);
    }
  }

  /// A variant of `when` that fallback to returning `null`
  ///
  /// It is equivalent to doing:
  /// ```dart
  /// switch (sealedClass) {
  ///   case Subclass(:final field):
  ///     return ...;
  ///   case _:
  ///     return null;
  /// }
  /// ```

  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(Int32List field0)? int,
    TResult? Function(Float32List field0)? float,
  }) {
    final _that = this;
    switch (_that) {
      case BarBeatData_Int() when int != null:
        return int(_that.field0);
      case BarBeatData_Float() when float != null:
        return float(_that.field0);
      case _:
        return null;
    }
  }
}

/// @nodoc

class BarBeatData_Int extends BarBeatData {
  const BarBeatData_Int(this.field0) : super._();

  @override
  final Int32List field0;

  /// Create a copy of BarBeatData
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $BarBeatData_IntCopyWith<BarBeatData_Int> get copyWith =>
      _$BarBeatData_IntCopyWithImpl<BarBeatData_Int>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is BarBeatData_Int &&
            const DeepCollectionEquality().equals(other.field0, field0));
  }

  @override
  int get hashCode =>
      Object.hash(runtimeType, const DeepCollectionEquality().hash(field0));

  @override
  String toString() {
    return 'BarBeatData.int(field0: $field0)';
  }
}

/// @nodoc
abstract mixin class $BarBeatData_IntCopyWith<$Res>
    implements $BarBeatDataCopyWith<$Res> {
  factory $BarBeatData_IntCopyWith(
          BarBeatData_Int value, $Res Function(BarBeatData_Int) _then) =
      _$BarBeatData_IntCopyWithImpl;
  @useResult
  $Res call({Int32List field0});
}

/// @nodoc
class _$BarBeatData_IntCopyWithImpl<$Res>
    implements $BarBeatData_IntCopyWith<$Res> {
  _$BarBeatData_IntCopyWithImpl(this._self, this._then);

  final BarBeatData_Int _self;
  final $Res Function(BarBeatData_Int) _then;

  /// Create a copy of BarBeatData
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  $Res call({
    Object? field0 = null,
  }) {
    return _then(BarBeatData_Int(
      null == field0
          ? _self.field0
          : field0 // ignore: cast_nullable_to_non_nullable
              as Int32List,
    ));
  }
}

/// @nodoc

class BarBeatData_Float extends BarBeatData {
  const BarBeatData_Float(this.field0) : super._();

  @override
  final Float32List field0;

  /// Create a copy of BarBeatData
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $BarBeatData_FloatCopyWith<BarBeatData_Float> get copyWith =>
      _$BarBeatData_FloatCopyWithImpl<BarBeatData_Float>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is BarBeatData_Float &&
            const DeepCollectionEquality().equals(other.field0, field0));
  }

  @override
  int get hashCode =>
      Object.hash(runtimeType, const DeepCollectionEquality().hash(field0));

  @override
  String toString() {
    return 'BarBeatData.float(field0: $field0)';
  }
}

/// @nodoc
abstract mixin class $BarBeatData_FloatCopyWith<$Res>
    implements $BarBeatDataCopyWith<$Res> {
  factory $BarBeatData_FloatCopyWith(
          BarBeatData_Float value, $Res Function(BarBeatData_Float) _then) =
      _$BarBeatData_FloatCopyWithImpl;
  @useResult
  $Res call({Float32List field0});
}

/// @nodoc
class _$BarBeatData_FloatCopyWithImpl<$Res>
    implements $BarBeatData_FloatCopyWith<$Res> {
  _$BarBeatData_FloatCopyWithImpl(this._self, this._then);

  final BarBeatData_Float _self;
  final $Res Function(BarBeatData_Float) _then;

  /// Create a copy of BarBeatData
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  $Res call({
    Object? field0 = null,
  }) {
    return _then(BarBeatData_Float(
      null == field0
          ? _self.field0
          : field0 // ignore: cast_nullable_to_non_nullable
              as Float32List,
    ));
  }
}

// dart format on
